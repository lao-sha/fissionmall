#![cfg_attr(not(feature = "std"), no_std)]

/// 机构运费模板模块
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config + scale_info::TypeInfo {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// 自定义区域最大长度
        #[pallet::constant]
        type MaxCustomAreaLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 运费模板结构
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct FreightTemplate<T: Config> {
        pub area: BoundedVec<u8, T::MaxCustomAreaLength>,
        pub first_weight: u64,
        pub first_weight_fee: u64,
        pub additional_weight_fee: u64,
        pub creator: T::AccountId,
    }

    /// 运费模板存储映射
    #[pallet::storage]
    #[pallet::storage_prefix = "FreightTemplates"]
    pub type FreightTemplates<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxCustomAreaLength>, // 主键：区域ID
        FreightTemplate<T>,                     // 值：运费模板
        OptionQuery,                            // 查询策略：如果键不存在，返回 None
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 运费模板已创建 [区域ID, 创建者]
        FreightTemplateCreated(BoundedVec<u8, T::MaxCustomAreaLength>, T::AccountId),
        /// 运费模板已更新 [区域ID]
        FreightTemplateUpdated(BoundedVec<u8, T::MaxCustomAreaLength>),
        /// 运费模板已删除 [区域ID]
        FreightTemplateDeleted(BoundedVec<u8, T::MaxCustomAreaLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 区域ID已存在
        AreaAlreadyExists,
        /// 运费模板不存在
        FreightTemplateNotFound,
        /// 无权操作此运费模板
        NotAuthorized,
        /// 字符串转换错误
        StringConversionError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新的运费模板
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_freight_template(
            origin: OriginFor<T>,
            area: Vec<u8>,
            first_weight: u64,
            first_weight_fee: u64,
            additional_weight_fee: u64,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_area = BoundedVec::<u8, T::MaxCustomAreaLength>::try_from(area)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查区域ID是否已存在
            ensure!(!FreightTemplates::<T>::contains_key(&bounded_area), Error::<T>::AreaAlreadyExists);
            
            // 创建运费模板
            let template = FreightTemplate {
                area: bounded_area.clone(),
                first_weight,
                first_weight_fee,
                additional_weight_fee,
                creator: who.clone(),
            };
            
            // 存储运费模板
            FreightTemplates::<T>::insert(&bounded_area, template);
            
            // 发出事件
            Self::deposit_event(Event::FreightTemplateCreated(bounded_area, who));
            
            Ok(())
        }
        
        /// 更新运费模板
        #[pallet::call_index(1)]
        #[pallet::weight(5_000)]
        pub fn update_freight_template(
            origin: OriginFor<T>,
            area: Vec<u8>,
            first_weight: Option<u64>,
            first_weight_fee: Option<u64>,
            additional_weight_fee: Option<u64>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_area = BoundedVec::<u8, T::MaxCustomAreaLength>::try_from(area)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新运费模板
            FreightTemplates::<T>::try_mutate(&bounded_area, |maybe_template| -> DispatchResult {
                let template = maybe_template.as_mut().ok_or(Error::<T>::FreightTemplateNotFound)?;
                
                // 检查权限（仅创建者可以更新）
                ensure!(template.creator == who, Error::<T>::NotAuthorized);
                
                // 更新各字段（如果提供）
                if let Some(weight) = first_weight {
                    template.first_weight = weight;
                }
                
                if let Some(fee) = first_weight_fee {
                    template.first_weight_fee = fee;
                }
                
                if let Some(add_fee) = additional_weight_fee {
                    template.additional_weight_fee = add_fee;
                }
                
                // 发出事件
                Self::deposit_event(Event::FreightTemplateUpdated(bounded_area.clone()));
                
                Ok(())
            })
        }
        
        /// 删除运费模板
        #[pallet::call_index(2)]
        #[pallet::weight(5_000)]
        pub fn delete_freight_template(
            origin: OriginFor<T>,
            area: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_area = BoundedVec::<u8, T::MaxCustomAreaLength>::try_from(area)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取运费模板并检查权限
            let template = FreightTemplates::<T>::get(&bounded_area)
                .ok_or(Error::<T>::FreightTemplateNotFound)?;
            
            ensure!(template.creator == who, Error::<T>::NotAuthorized);
            
            // 删除运费模板
            FreightTemplates::<T>::remove(&bounded_area);
            
            // 发出事件
            Self::deposit_event(Event::FreightTemplateDeleted(bounded_area));
            
            Ok(())
        }
    }
} 