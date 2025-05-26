#![cfg_attr(not(feature = "std"), no_std)]

/// 订单管理模块
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::{Get, ConstU32}};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config + scale_info::TypeInfo {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// 订单编码最大长度
        #[pallet::constant]
        type MaxOrderCodeLength: Get<u32>;
        
        /// 会员编码最大长度
        #[pallet::constant]
        type MaxMemberCodeLength: Get<u32>;
        
        /// 机构ID最大长度
        #[pallet::constant]
        type MaxInstitutionIdLength: Get<u32>;
        
        /// 订单项最大数量
        #[pallet::constant]
        type MaxOrderItems: Get<u32>;
        
        /// 快递公司名称最大长度
        #[pallet::constant]
        type MaxExpressCompanyLength: Get<u32>;
        
        /// 快递单号最大长度
        #[pallet::constant]
        type MaxExpressNumberLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 订单状态枚举
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum OrderStatus {
        Pending = 0,    // 待支付
        Paid = 1,       // 已支付
        Delivered = 2,  // 已发货
        Cancelled = 3,  // 已取消
        Completed = 4,  // 已完成
        Refunded = 5,   // 已退款
    }

    /// 联系信息结构
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ContactInformation {
        pub phone: Option<BoundedVec<u8, ConstU32<32>>>,    // 电话号码
        pub email: Option<BoundedVec<u8, ConstU32<128>>>,   // 邮箱
        pub address: Option<BoundedVec<u8, ConstU32<512>>>, // 地址
    }

    /// 订单中的商品项
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct OrderItem {
        pub product_code: BoundedVec<u8, ConstU32<64>>, // 商品ID
        pub quantity: u32,         // 商品数量
        pub price_per_unit: u32,   // 单价，单位为人民币
        pub weight: u32,           // 商品重量
    }

    /// 用户的订单
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Order<T: Config> {
        pub order_code: BoundedVec<u8, T::MaxOrderCodeLength>,           // 订单ID
        pub member_code: BoundedVec<u8, T::MaxMemberCodeLength>,         // 用户ID
        pub institution_code: BoundedVec<u8, T::MaxInstitutionIdLength>, // 所属机构
        pub status: OrderStatus,                                         // 订单状态
        pub created_time: BlockNumberFor<T>,                             // 创建时间
        pub updated_time: BlockNumberFor<T>,                             // 更新时间
        pub total_amount: u32,                                          // 总金额
        pub total_weight: u32,                                          // 总重量
        pub freight: u32,                                               // 运费
        pub contact_information: ContactInformation,                     // 联系信息
        pub items: BoundedVec<OrderItem, T::MaxOrderItems>,            // 订单明细
        pub express_company: BoundedVec<u8, T::MaxExpressCompanyLength>, // 快递公司名称
        pub express_number: BoundedVec<u8, T::MaxExpressNumberLength>,   // 快递单号
        pub creator: T::AccountId,                                       // 创建者
    }

    /// 订单存储映射
    #[pallet::storage]
    #[pallet::storage_prefix = "Orders"]
    pub type Orders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxOrderCodeLength>,  // 主键：订单编码
        Order<T>,                               // 值：订单信息
        OptionQuery,                            // 查询策略：如果键不存在，返回 None
    >;

    /// 用户订单索引
    #[pallet::storage]
    #[pallet::storage_prefix = "UserOrders"]
    pub type UserOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxMemberCodeLength>,                     // 主键：用户编码
        BoundedVec<BoundedVec<u8, T::MaxOrderCodeLength>, ConstU32<1000>>, // 值：订单编码列表
        ValueQuery,                                                  // 查询策略：如果键不存在，返回空列表
    >;

    /// 机构订单索引
    #[pallet::storage]
    #[pallet::storage_prefix = "InstitutionOrders"]
    pub type InstitutionOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionIdLength>,                  // 主键：机构编码
        BoundedVec<BoundedVec<u8, T::MaxOrderCodeLength>, ConstU32<10000>>, // 值：订单编码列表
        ValueQuery,                                                  // 查询策略：如果键不存在，返回空列表
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 订单已创建 [订单编码, 创建者]
        OrderCreated(BoundedVec<u8, T::MaxOrderCodeLength>, T::AccountId),
        /// 订单状态已更新 [订单编码, 新状态(u8)]
        OrderStatusUpdated(BoundedVec<u8, T::MaxOrderCodeLength>, u8),
        /// 订单快递信息已更新 [订单编码]
        OrderExpressInfoUpdated(BoundedVec<u8, T::MaxOrderCodeLength>),
        /// 订单已取消 [订单编码]
        OrderCancelled(BoundedVec<u8, T::MaxOrderCodeLength>),
        /// 订单已删除 [订单编码]
        OrderDeleted(BoundedVec<u8, T::MaxOrderCodeLength>),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 订单编码已存在
        OrderCodeAlreadyExists,
        /// 订单不存在
        OrderNotFound,
        /// 无权操作此订单
        NotAuthorized,
        /// 字符串转换错误
        StringConversionError,
        /// 无效状态
        InvalidStatus,
        /// 订单项为空
        EmptyOrderItems,
        /// 订单项数量超过限制
        TooManyOrderItems,
        /// 无效的订单状态转换
        InvalidStatusTransition,
        /// 用户订单列表已满
        UserOrderListFull,
        /// 机构订单列表已满
        InstitutionOrderListFull,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新订单
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_order(
            origin: OriginFor<T>,
            order_code: Vec<u8>,
            member_code: Vec<u8>,
            institution_code: Vec<u8>,
            freight: u32,
            phone: Option<Vec<u8>>,
            email: Option<Vec<u8>>,
            address: Option<Vec<u8>>,
            items: Vec<(Vec<u8>, u32, u32, u32)>, // (product_code, quantity, price_per_unit, weight)
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_order_code = BoundedVec::<u8, T::MaxOrderCodeLength>::try_from(order_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查订单编码是否已存在
            ensure!(!Orders::<T>::contains_key(&bounded_order_code), Error::<T>::OrderCodeAlreadyExists);
            
            // 检查订单项不为空
            ensure!(!items.is_empty(), Error::<T>::EmptyOrderItems);
            
            // 转换其他字段为边界向量
            let bounded_member_code = BoundedVec::<u8, T::MaxMemberCodeLength>::try_from(member_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 创建联系信息
            let contact_information = ContactInformation {
                phone: match phone {
                    Some(p) => Some(BoundedVec::<u8, ConstU32<32>>::try_from(p)
                        .map_err(|_| Error::<T>::StringConversionError)?),
                    None => None,
                },
                email: match email {
                    Some(e) => Some(BoundedVec::<u8, ConstU32<128>>::try_from(e)
                        .map_err(|_| Error::<T>::StringConversionError)?),
                    None => None,
                },
                address: match address {
                    Some(a) => Some(BoundedVec::<u8, ConstU32<512>>::try_from(a)
                        .map_err(|_| Error::<T>::StringConversionError)?),
                    None => None,
                },
            };
            
            // 转换订单项列表
            let mut order_items = Vec::new();
            let mut total_amount = 0u32;
            let mut total_weight = 0u32;
            
            for (product_code, quantity, price_per_unit, weight) in items {
                let item = OrderItem {
                    product_code: BoundedVec::<u8, ConstU32<64>>::try_from(product_code)
                        .map_err(|_| Error::<T>::StringConversionError)?,
                    quantity,
                    price_per_unit,
                    weight,
                };
                
                total_amount = total_amount.saturating_add(price_per_unit.saturating_mul(quantity));
                total_weight = total_weight.saturating_add(weight.saturating_mul(quantity));
                
                order_items.push(item);
            }
            
            let bounded_items = BoundedVec::<OrderItem, T::MaxOrderItems>::try_from(order_items)
                .map_err(|_| Error::<T>::TooManyOrderItems)?;
            
            // 加上运费
            total_amount = total_amount.saturating_add(freight);
            
            // 创建订单
            let order = Order {
                order_code: bounded_order_code.clone(),
                member_code: bounded_member_code.clone(),
                institution_code: bounded_institution_code.clone(),
                status: OrderStatus::Pending,
                created_time: frame_system::Pallet::<T>::block_number(),
                updated_time: frame_system::Pallet::<T>::block_number(),
                total_amount,
                total_weight,
                freight,
                contact_information,
                items: bounded_items,
                express_company: BoundedVec::default(),
                express_number: BoundedVec::default(),
                creator: who.clone(),
            };
            
            // 存储订单
            Orders::<T>::insert(&bounded_order_code, &order);
            
            // 更新用户订单索引
            UserOrders::<T>::try_mutate(&bounded_member_code, |orders| -> DispatchResult {
                orders.try_push(bounded_order_code.clone())
                    .map_err(|_| Error::<T>::UserOrderListFull)?;
                Ok(())
            })?;
            
            // 更新机构订单索引
            InstitutionOrders::<T>::try_mutate(&bounded_institution_code, |orders| -> DispatchResult {
                orders.try_push(bounded_order_code.clone())
                    .map_err(|_| Error::<T>::InstitutionOrderListFull)?;
                Ok(())
            })?;
            
            // 发出事件
            Self::deposit_event(Event::OrderCreated(bounded_order_code, who));
            
            Ok(())
        }
        
        /// 更新订单状态
        #[pallet::call_index(1)]
        #[pallet::weight(5_000)]
        pub fn update_order_status(
            origin: OriginFor<T>,
            order_code: Vec<u8>,
            status: u8,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_order_code = BoundedVec::<u8, T::MaxOrderCodeLength>::try_from(order_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新订单
            Orders::<T>::try_mutate(&bounded_order_code, |maybe_order| -> DispatchResult {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                
                // 检查权限（仅创建者可以更新）
                ensure!(order.creator == who, Error::<T>::NotAuthorized);
                
                // 将 u8 转换为 OrderStatus
                let new_status = match status {
                    0 => OrderStatus::Pending,
                    1 => OrderStatus::Paid,
                    2 => OrderStatus::Delivered,
                    3 => OrderStatus::Cancelled,
                    4 => OrderStatus::Completed,
                    5 => OrderStatus::Refunded,
                    _ => return Err(Error::<T>::InvalidStatus.into()),
                };
                
                // 检查状态转换是否有效
                Self::validate_status_transition(&order.status, &new_status)?;
                
                // 更新状态和时间
                order.status = new_status;
                order.updated_time = frame_system::Pallet::<T>::block_number();
                
                // 发出事件
                Self::deposit_event(Event::OrderStatusUpdated(bounded_order_code.clone(), status));
                
                Ok(())
            })
        }
        
        /// 更新订单快递信息
        #[pallet::call_index(2)]
        #[pallet::weight(5_000)]
        pub fn update_express_info(
            origin: OriginFor<T>,
            order_code: Vec<u8>,
            express_company: Vec<u8>,
            express_number: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_order_code = BoundedVec::<u8, T::MaxOrderCodeLength>::try_from(order_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_express_company = BoundedVec::<u8, T::MaxExpressCompanyLength>::try_from(express_company)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_express_number = BoundedVec::<u8, T::MaxExpressNumberLength>::try_from(express_number)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新订单
            Orders::<T>::try_mutate(&bounded_order_code, |maybe_order| -> DispatchResult {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                
                // 检查权限（仅创建者可以更新）
                ensure!(order.creator == who, Error::<T>::NotAuthorized);
                
                // 更新快递信息
                order.express_company = bounded_express_company;
                order.express_number = bounded_express_number;
                order.updated_time = frame_system::Pallet::<T>::block_number();
                
                // 发出事件
                Self::deposit_event(Event::OrderExpressInfoUpdated(bounded_order_code.clone()));
                
                Ok(())
            })
        }
        
        /// 取消订单
        #[pallet::call_index(3)]
        #[pallet::weight(5_000)]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order_code: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_order_code = BoundedVec::<u8, T::MaxOrderCodeLength>::try_from(order_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新订单
            Orders::<T>::try_mutate(&bounded_order_code, |maybe_order| -> DispatchResult {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                
                // 检查权限（仅创建者可以取消）
                ensure!(order.creator == who, Error::<T>::NotAuthorized);
                
                // 只有待支付或已支付的订单可以取消
                ensure!(
                    matches!(order.status, OrderStatus::Pending | OrderStatus::Paid),
                    Error::<T>::InvalidStatusTransition
                );
                
                // 更新状态
                order.status = OrderStatus::Cancelled;
                order.updated_time = frame_system::Pallet::<T>::block_number();
                
                // 发出事件
                Self::deposit_event(Event::OrderCancelled(bounded_order_code.clone()));
                
                Ok(())
            })
        }
        
        /// 删除订单
        #[pallet::call_index(4)]
        #[pallet::weight(5_000)]
        pub fn delete_order(
            origin: OriginFor<T>,
            order_code: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_order_code = BoundedVec::<u8, T::MaxOrderCodeLength>::try_from(order_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取订单并检查权限
            let order = Orders::<T>::get(&bounded_order_code)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            ensure!(order.creator == who, Error::<T>::NotAuthorized);
            
            // 从用户订单索引中移除
            UserOrders::<T>::mutate(&order.member_code, |orders| {
                orders.retain(|code| code != &bounded_order_code);
            });
            
            // 从机构订单索引中移除
            InstitutionOrders::<T>::mutate(&order.institution_code, |orders| {
                orders.retain(|code| code != &bounded_order_code);
            });
            
            // 删除订单
            Orders::<T>::remove(&bounded_order_code);
            
            // 发出事件
            Self::deposit_event(Event::OrderDeleted(bounded_order_code));
            
            Ok(())
        }
    }
    
    // 辅助函数
    impl<T: Config> Pallet<T> {
        /// 验证状态转换是否有效
        fn validate_status_transition(from: &OrderStatus, to: &OrderStatus) -> DispatchResult {
            use OrderStatus::*;
            
            let valid_transition = match (from, to) {
                // 待支付可以转换为已支付或已取消
                (Pending, Paid) | (Pending, Cancelled) => true,
                // 已支付可以转换为已发货、已退款或已取消
                (Paid, Delivered) | (Paid, Refunded) | (Paid, Cancelled) => true,
                // 已发货可以转换为已完成
                (Delivered, Completed) => true,
                // 已完成可以转换为已退款
                (Completed, Refunded) => true,
                // 其他转换无效
                _ => false,
            };
            
            ensure!(valid_transition, Error::<T>::InvalidStatusTransition);
            Ok(())
        }
    }
} 