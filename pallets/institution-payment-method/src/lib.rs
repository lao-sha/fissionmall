#![cfg_attr(not(feature = "std"), no_std)]

/// 机构支付方式管理模块
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;

    /// 支付方式信息最大长度类型
    pub type MaxPaymentLengthType = ConstU32<256>;

    #[pallet::config]
    pub trait Config: frame_system::Config + scale_info::TypeInfo {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// 机构ID最大长度
        #[pallet::constant]
        type MaxInstitutionIdLength: Get<u32>;
        
        /// 支付方式信息最大长度
        #[pallet::constant]
        type MaxPaymentLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 支付方式结构
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct PaymentMethod<T: Config> {
        pub wechat: Option<BoundedVec<u8, T::MaxPaymentLength>>,
        pub alipay: Option<BoundedVec<u8, T::MaxPaymentLength>>,
        pub token: Option<BoundedVec<u8, T::MaxPaymentLength>>,
        pub other: Option<BoundedVec<u8, T::MaxPaymentLength>>,
    }

    /// 支付方式存储映射
    #[pallet::storage]
    #[pallet::storage_prefix = "PaymentMethods"]
    pub type PaymentMethods<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionIdLength>, // 主键：机构 ID
        PaymentMethod<T>,                        // 值：支付方式
        OptionQuery,                               // 查询策略：如果键不存在，返回 None
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 支付方式已创建 [机构ID, 创建者]
        PaymentMethodCreated(BoundedVec<u8, T::MaxInstitutionIdLength>, T::AccountId),
        /// 支付方式已更新 [机构ID]
        PaymentMethodUpdated(BoundedVec<u8, T::MaxInstitutionIdLength>),
        /// 支付方式已删除 [机构ID]
        PaymentMethodDeleted(BoundedVec<u8, T::MaxInstitutionIdLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 机构ID不存在
        InstitutionNotFound,
        /// 支付方式已存在
        PaymentMethodAlreadyExists,
        /// 支付方式不存在
        PaymentMethodNotFound,
        /// 无权操作此支付方式
        NotAuthorized,
        /// 字符串转换错误
        StringConversionError,
        /// 至少需要设置一种支付方式
        AtLeastOnePaymentMethodRequired,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建或更新机构的支付方式
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn set_payment_method(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
            wechat: Option<Vec<u8>>,
            alipay: Option<Vec<u8>>,
            token: Option<Vec<u8>>,
            other: Option<Vec<u8>>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 确保至少设置了一种支付方式
            ensure!(
                wechat.is_some() || alipay.is_some() || token.is_some() || other.is_some(),
                Error::<T>::AtLeastOnePaymentMethodRequired
            );
            
            // 转换支付方式字段为边界向量
            let bounded_wechat = match wechat {
                Some(w) => Some(BoundedVec::<u8, T::MaxPaymentLength>::try_from(w)
                    .map_err(|_| Error::<T>::StringConversionError)?),
                None => None,
            };
            
            let bounded_alipay = match alipay {
                Some(a) => Some(BoundedVec::<u8, T::MaxPaymentLength>::try_from(a)
                    .map_err(|_| Error::<T>::StringConversionError)?),
                None => None,
            };
            
            let bounded_token = match token {
                Some(t) => Some(BoundedVec::<u8, T::MaxPaymentLength>::try_from(t)
                    .map_err(|_| Error::<T>::StringConversionError)?),
                None => None,
            };
            
            let bounded_other = match other {
                Some(o) => Some(BoundedVec::<u8, T::MaxPaymentLength>::try_from(o)
                    .map_err(|_| Error::<T>::StringConversionError)?),
                None => None,
            };
            
            // 创建支付方式结构
            let payment_method = PaymentMethod {
                wechat: bounded_wechat,
                alipay: bounded_alipay,
                token: bounded_token,
                other: bounded_other,
            };
            
            // 检查是否已存在支付方式
            let is_update = PaymentMethods::<T>::contains_key(&bounded_id);
            
            // 存储支付方式
            PaymentMethods::<T>::insert(&bounded_id, payment_method);
            
            // 发出相应事件
            if is_update {
                Self::deposit_event(Event::PaymentMethodUpdated(bounded_id));
            } else {
                Self::deposit_event(Event::PaymentMethodCreated(bounded_id, who));
            }
            
            Ok(())
        }
        
        /// 删除机构的支付方式
        #[pallet::call_index(1)]
        #[pallet::weight(5_000)]
        pub fn remove_payment_method(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let _who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查支付方式是否存在
            ensure!(PaymentMethods::<T>::contains_key(&bounded_id), Error::<T>::PaymentMethodNotFound);
            
            // 删除支付方式
            PaymentMethods::<T>::remove(&bounded_id);
            
            // 发出事件
            Self::deposit_event(Event::PaymentMethodDeleted(bounded_id));
            
            Ok(())
        }
        
        /// 更新特定支付方式字段
        #[pallet::call_index(2)]
        #[pallet::weight(7_000)]
        pub fn update_payment_field(
            origin: OriginFor<T>,
            institution_id: Vec<u8>,
            field_type: u8, // 0: wechat, 1: alipay, 2: token, 3: other
            value: Option<Vec<u8>>,
        ) -> DispatchResult {
            // 确认调用者身份
            let _who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_id = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_id)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 转换值为边界向量
            let bounded_value = match value {
                Some(v) => Some(BoundedVec::<u8, T::MaxPaymentLength>::try_from(v)
                    .map_err(|_| Error::<T>::StringConversionError)?),
                None => None,
            };
            
            // 更新支付方式
            PaymentMethods::<T>::try_mutate(&bounded_id, |maybe_payment| -> DispatchResult {
                let payment = maybe_payment.as_mut().ok_or(Error::<T>::PaymentMethodNotFound)?;
                
                match field_type {
                    0 => payment.wechat = bounded_value,
                    1 => payment.alipay = bounded_value,
                    2 => payment.token = bounded_value,
                    3 => payment.other = bounded_value,
                    _ => return Err(Error::<T>::StringConversionError.into()),
                }
                
                Ok(())
            })?;
            
            // 发出事件
            Self::deposit_event(Event::PaymentMethodUpdated(bounded_id));
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 获取机构的支付方式
        pub fn get_payment_method(institution_id: &BoundedVec<u8, T::MaxInstitutionIdLength>) -> Option<PaymentMethod<T>> {
            PaymentMethods::<T>::get(institution_id)
        }
        
        /// 检查机构是否设置了支付方式
        pub fn has_payment_method(institution_id: &BoundedVec<u8, T::MaxInstitutionIdLength>) -> bool {
            PaymentMethods::<T>::contains_key(institution_id)
        }
    }
} 