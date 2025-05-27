#![cfg_attr(not(feature = "std"), no_std)]

/// 商品管理模块
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_runtime::Perbill;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;
    use codec::{Decode, Encode};

    #[pallet::config]
    pub trait Config: frame_system::Config + scale_info::TypeInfo {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// 商品代码最大长度
        #[pallet::constant]
        type MaxProductCodeLength: Get<u32>;
        
        /// 机构代码最大长度
        #[pallet::constant]
        type MaxInstitutionCodeLength: Get<u32>;
        
        /// 名称最大长度
        #[pallet::constant]
        type MaxNameLength: Get<u32>;
        
        /// 分类最大长度
        #[pallet::constant]
        type MaxCategoryLength: Get<u32>;
        
        /// 品牌最大长度
        #[pallet::constant]
        type MaxBrandLength: Get<u32>;
        
        /// 授权用户组名称最大长度
        #[pallet::constant]
        type MaxAuthorizedMemberGroup: Get<u32>;
        
        /// 授权用户组最大数量
        #[pallet::constant]
        type MaxAuthorizedGroups: Get<u32>;
        
        /// 描述最大长度
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;
        
        /// 图片URL最大长度
        #[pallet::constant]
        type MaxImageUrlLength: Get<u32>;
        
        /// 详情图最大数量
        #[pallet::constant]
        type MaxDetailImages: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 商品状态枚举
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum ProductStatus {
        Available = 0,   // 上架
        Unavailable = 1, // 下架
    }

    /// 商品信息结构体
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ProductInfo<T: Config> {
        pub product_name: BoundedVec<u8, T::MaxNameLength>,                                                              // 商品名称
        pub category: BoundedVec<u8, T::MaxCategoryLength>,                                                              // 分类
        pub brand: BoundedVec<u8, T::MaxBrandLength>,                                                                    // 商品品牌
        pub authorized_member_groups: BoundedVec<BoundedVec<u8, T::MaxAuthorizedMemberGroup>, T::MaxAuthorizedGroups>,   // 授权用户组
        pub original_price: u64,                                                                                         // 原价
        pub current_price: u64,                                                                                          // 现价
        pub description: BoundedVec<u8, T::MaxDescriptionLength>,                                                        // 商品描述
        pub main_image: BoundedVec<u8, T::MaxImageUrlLength>,                                                            // 主图
        pub detail_images: BoundedVec<BoundedVec<u8, T::MaxImageUrlLength>, T::MaxDetailImages>,                         // 详情图
        pub stock_quantity: u32,                                                                                         // 库存数量
        pub sales_quantity: u32,                                                                                         // 销售数量
        pub weight: u32,                                                                                                 // 重量
        pub status: ProductStatus,                                                                                       // 商品状态
        pub profit_ratio: Perbill,                                                                                       // 分润比例
        pub created_date: BlockNumberFor<T>,                                                                             // 创建日期
        pub creator: T::AccountId,                                                                                       // 创建者
    }

    /// 存储商品信息的映射，主键为商品 ID 和机构 ID
    #[pallet::storage]
    #[pallet::getter(fn products)]
    pub type Products<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxProductCodeLength>,     // 商品代码
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionCodeLength>, // 所属机构代码
        ProductInfo<T>,                              // 商品信息
        OptionQuery,                                 // 查询策略：如果键不存在，返回 None
    >;

    /// 机构商品索引
    #[pallet::storage]
    #[pallet::getter(fn institution_products)]
    pub type InstitutionProducts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionCodeLength>,                               // 机构代码
        BoundedVec<BoundedVec<u8, T::MaxProductCodeLength>, ConstU32<10000>>,      // 商品代码列表
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 商品已创建 [商品代码, 机构代码, 创建者]
        ProductCreated(BoundedVec<u8, T::MaxProductCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, T::AccountId),
        /// 商品信息已更新 [商品代码, 机构代码]
        ProductUpdated(BoundedVec<u8, T::MaxProductCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>),
        /// 商品状态已更新 [商品代码, 机构代码, 新状态(u8)]
        ProductStatusUpdated(BoundedVec<u8, T::MaxProductCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, u8),
        /// 商品库存已更新 [商品代码, 机构代码, 新库存]
        ProductStockUpdated(BoundedVec<u8, T::MaxProductCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, u32),
        /// 商品已删除 [商品代码, 机构代码]
        ProductDeleted(BoundedVec<u8, T::MaxProductCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 商品已存在
        ProductAlreadyExists,
        /// 商品不存在
        ProductNotFound,
        /// 无权操作此商品
        NotAuthorized,
        /// 字符串转换错误
        StringConversionError,
        /// 授权用户组数量超过限制
        TooManyAuthorizedGroups,
        /// 详情图数量超过限制
        TooManyDetailImages,
        /// 无效的价格（现价高于原价）
        InvalidPrice,
        /// 无效的分润比例
        InvalidProfitRatio,
        /// 机构商品列表已满
        InstitutionProductListFull,
        /// 库存不足
        InsufficientStock,
        /// 无效的状态
        InvalidStatus,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新商品
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_product(
            origin: OriginFor<T>,
            product_code: Vec<u8>,
            institution_code: Vec<u8>,
            product_name: Vec<u8>,
            category: Vec<u8>,
            brand: Vec<u8>,
            authorized_member_groups: Vec<Vec<u8>>,
            original_price: u64,
            current_price: u64,
            description: Vec<u8>,
            main_image: Vec<u8>,
            detail_images: Vec<Vec<u8>>,
            stock_quantity: u32,
            weight: u32,
            profit_ratio: Perbill,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_product_code = BoundedVec::<u8, T::MaxProductCodeLength>::try_from(product_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查商品是否已存在
            ensure!(!Products::<T>::contains_key(&bounded_product_code, &bounded_institution_code), 
                Error::<T>::ProductAlreadyExists);
            
            // 验证价格
            ensure!(current_price <= original_price, Error::<T>::InvalidPrice);
            
            // 转换其他字段
            let bounded_product_name = BoundedVec::<u8, T::MaxNameLength>::try_from(product_name)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_category = BoundedVec::<u8, T::MaxCategoryLength>::try_from(category)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_brand = BoundedVec::<u8, T::MaxBrandLength>::try_from(brand)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 转换授权用户组
            let mut bounded_groups = Vec::new();
            for group in authorized_member_groups {
                let bounded_group = BoundedVec::<u8, T::MaxAuthorizedMemberGroup>::try_from(group)
                    .map_err(|_| Error::<T>::StringConversionError)?;
                bounded_groups.push(bounded_group);
            }
            let bounded_authorized_groups = BoundedVec::<BoundedVec<u8, T::MaxAuthorizedMemberGroup>, T::MaxAuthorizedGroups>::try_from(bounded_groups)
                .map_err(|_| Error::<T>::TooManyAuthorizedGroups)?;
            
            let bounded_description = BoundedVec::<u8, T::MaxDescriptionLength>::try_from(description)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_main_image = BoundedVec::<u8, T::MaxImageUrlLength>::try_from(main_image)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 转换详情图
            let mut bounded_detail_images = Vec::new();
            for image in detail_images {
                let bounded_image = BoundedVec::<u8, T::MaxImageUrlLength>::try_from(image)
                    .map_err(|_| Error::<T>::StringConversionError)?;
                bounded_detail_images.push(bounded_image);
            }
            let bounded_detail_images = BoundedVec::<BoundedVec<u8, T::MaxImageUrlLength>, T::MaxDetailImages>::try_from(bounded_detail_images)
                .map_err(|_| Error::<T>::TooManyDetailImages)?;
            
            // 创建商品信息
            let product_info = ProductInfo {
                product_name: bounded_product_name,
                category: bounded_category,
                brand: bounded_brand,
                authorized_member_groups: bounded_authorized_groups,
                original_price,
                current_price,
                description: bounded_description,
                main_image: bounded_main_image,
                detail_images: bounded_detail_images,
                stock_quantity,
                sales_quantity: 0,
                weight,
                status: ProductStatus::Available,
                profit_ratio,
                created_date: frame_system::Pallet::<T>::block_number(),
                creator: who.clone(),
            };
            
            // 存储商品信息
            Products::<T>::insert(&bounded_product_code, &bounded_institution_code, &product_info);
            
            // 更新机构商品索引
            InstitutionProducts::<T>::try_mutate(&bounded_institution_code, |products| -> DispatchResult {
                products.try_push(bounded_product_code.clone())
                    .map_err(|_| Error::<T>::InstitutionProductListFull)?;
                Ok(())
            })?;
            
            // 发出事件
            Self::deposit_event(Event::ProductCreated(bounded_product_code, bounded_institution_code, who));
            
            Ok(())
        }
        
        /// 更新商品信息
        #[pallet::call_index(1)]
        #[pallet::weight(8_000)]
        pub fn update_product_info(
            origin: OriginFor<T>,
            product_code: Vec<u8>,
            institution_code: Vec<u8>,
            product_name: Option<Vec<u8>>,
            category: Option<Vec<u8>>,
            brand: Option<Vec<u8>>,
            original_price: Option<u64>,
            current_price: Option<u64>,
            description: Option<Vec<u8>>,
            main_image: Option<Vec<u8>>,
            weight: Option<u32>,
            profit_ratio: Option<Perbill>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_product_code = BoundedVec::<u8, T::MaxProductCodeLength>::try_from(product_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新商品信息
            Products::<T>::try_mutate(&bounded_product_code, &bounded_institution_code, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;
                
                // 检查权限
                ensure!(product.creator == who, Error::<T>::NotAuthorized);
                
                // 更新各字段（如果提供）
                if let Some(name) = product_name {
                    product.product_name = BoundedVec::<u8, T::MaxNameLength>::try_from(name)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(cat) = category {
                    product.category = BoundedVec::<u8, T::MaxCategoryLength>::try_from(cat)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(br) = brand {
                    product.brand = BoundedVec::<u8, T::MaxBrandLength>::try_from(br)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(op) = original_price {
                    product.original_price = op;
                }
                
                if let Some(cp) = current_price {
                    product.current_price = cp;
                }
                
                // 验证价格
                ensure!(product.current_price <= product.original_price, Error::<T>::InvalidPrice);
                
                if let Some(desc) = description {
                    product.description = BoundedVec::<u8, T::MaxDescriptionLength>::try_from(desc)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(img) = main_image {
                    product.main_image = BoundedVec::<u8, T::MaxImageUrlLength>::try_from(img)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(w) = weight {
                    product.weight = w;
                }
                
                if let Some(pr) = profit_ratio {
                    product.profit_ratio = pr;
                }
                
                // 发出事件
                Self::deposit_event(Event::ProductUpdated(bounded_product_code.clone(), bounded_institution_code.clone()));
                
                Ok(())
            })
        }
        
        /// 更新商品状态
        #[pallet::call_index(2)]
        #[pallet::weight(5_000)]
        pub fn update_product_status(
            origin: OriginFor<T>,
            product_code: Vec<u8>,
            institution_code: Vec<u8>,
            status: u8,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_product_code = BoundedVec::<u8, T::MaxProductCodeLength>::try_from(product_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新商品状态
            Products::<T>::try_mutate(&bounded_product_code, &bounded_institution_code, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;
                
                // 检查权限
                ensure!(product.creator == who, Error::<T>::NotAuthorized);
                
                // 转换状态
                let new_status = match status {
                    0 => ProductStatus::Available,
                    1 => ProductStatus::Unavailable,
                    _ => return Err(Error::<T>::InvalidStatus.into()),
                };
                
                // 更新状态
                product.status = new_status;
                
                // 发出事件
                Self::deposit_event(Event::ProductStatusUpdated(
                    bounded_product_code.clone(), 
                    bounded_institution_code.clone(), 
                    status
                ));
                
                Ok(())
            })
        }
        
        /// 更新商品库存
        #[pallet::call_index(3)]
        #[pallet::weight(5_000)]
        pub fn update_stock(
            origin: OriginFor<T>,
            product_code: Vec<u8>,
            institution_code: Vec<u8>,
            new_stock: u32,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_product_code = BoundedVec::<u8, T::MaxProductCodeLength>::try_from(product_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新商品库存
            Products::<T>::try_mutate(&bounded_product_code, &bounded_institution_code, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;
                
                // 检查权限
                ensure!(product.creator == who, Error::<T>::NotAuthorized);
                
                // 更新库存
                product.stock_quantity = new_stock;
                
                // 发出事件
                Self::deposit_event(Event::ProductStockUpdated(
                    bounded_product_code.clone(), 
                    bounded_institution_code.clone(), 
                    new_stock
                ));
                
                Ok(())
            })
        }
        
        /// 删除商品
        #[pallet::call_index(4)]
        #[pallet::weight(5_000)]
        pub fn delete_product(
            origin: OriginFor<T>,
            product_code: Vec<u8>,
            institution_code: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_product_code = BoundedVec::<u8, T::MaxProductCodeLength>::try_from(product_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取商品并检查权限
            let product = Products::<T>::get(&bounded_product_code, &bounded_institution_code)
                .ok_or(Error::<T>::ProductNotFound)?;
            
            ensure!(product.creator == who, Error::<T>::NotAuthorized);
            
            // 从机构商品索引中移除
            InstitutionProducts::<T>::mutate(&bounded_institution_code, |products| {
                products.retain(|code| code != &bounded_product_code);
            });
            
            // 删除商品
            Products::<T>::remove(&bounded_product_code, &bounded_institution_code);
            
            // 发出事件
            Self::deposit_event(Event::ProductDeleted(bounded_product_code, bounded_institution_code));
            
            Ok(())
        }
        
        /// 购买商品（减少库存，增加销售数量）
        #[pallet::call_index(5)]
        #[pallet::weight(5_000)]
        pub fn purchase_product(
            origin: OriginFor<T>,
            product_code: Vec<u8>,
            institution_code: Vec<u8>,
            quantity: u32,
        ) -> DispatchResult {
            // 确认调用者身份
            let _who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_product_code = BoundedVec::<u8, T::MaxProductCodeLength>::try_from(product_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新商品信息
            Products::<T>::try_mutate(&bounded_product_code, &bounded_institution_code, |maybe_product| -> DispatchResult {
                let product = maybe_product.as_mut().ok_or(Error::<T>::ProductNotFound)?;
                
                // 检查商品状态
                ensure!(product.status == ProductStatus::Available, Error::<T>::ProductNotFound);
                
                // 检查库存
                ensure!(product.stock_quantity >= quantity, Error::<T>::InsufficientStock);
                
                // 更新库存和销售数量
                product.stock_quantity = product.stock_quantity.saturating_sub(quantity);
                product.sales_quantity = product.sales_quantity.saturating_add(quantity);
                
                Ok(())
            })
        }
    }
} 