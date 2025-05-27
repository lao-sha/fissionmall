#![cfg_attr(not(feature = "std"), no_std)]

/// C2C 订单管理模块
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_std::prelude::*;
    use sp_std::vec::Vec;
    use codec::{Decode, Encode};

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
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 订单状态枚举
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum OrderStatus {
        Pending = 0,    // 待支付
        Paid = 1,       // 已支付
        Delivered = 2,  // 已发货
        Notarizing = 3, // 公证中
        Cancelled = 4,  // 已取消
        Completed = 5,  // 已完成
    }

    impl Default for OrderStatus {
        fn default() -> Self {
            OrderStatus::Pending
        }
    }

    /// 订单方向枚举
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum OrderDirection {
        UserSell = 0, // 用户出售
        UserBuy = 1,  // 用户购买
    }

    impl Default for OrderDirection {
        fn default() -> Self {
            OrderDirection::UserBuy
        }
    }

    /// 用户的订单
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, Default)]
    pub struct Order<T: Config> {
        pub order_code: BoundedVec<u8, T::MaxOrderCodeLength>,           // 订单ID
        pub member_code: BoundedVec<u8, T::MaxMemberCodeLength>,         // 用户ID
        pub institution_code: BoundedVec<u8, T::MaxInstitutionIdLength>, // 所属机构
        pub status: OrderStatus,                                         // 订单状态
        pub direction: OrderDirection,                                   // 订单方向
        pub created_time: BlockNumberFor<T>,                             // 创建时间
        pub updated_time: BlockNumberFor<T>,                             // 更新时间
        pub transaction_amount: u128,                                    // 交易金额
        pub total_amount: u128,                                          // 总金额
        pub creator: T::AccountId,                                       // 创建者
    }

    /// 订单存储映射
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxOrderCodeLength>,  // 主键：订单编码
        Order<T>,                               // 值：订单信息
        OptionQuery,                            // 查询策略：如果键不存在，返回 None
    >;

    /// 用户订单索引
    #[pallet::storage]
    #[pallet::getter(fn user_orders)]
    pub type UserOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxMemberCodeLength>,                              // 主键：用户编码
        BoundedVec<BoundedVec<u8, T::MaxOrderCodeLength>, ConstU32<1000>>,   // 值：订单编码列表
        ValueQuery,                                                          // 查询策略：如果键不存在，返回空列表
    >;

    /// 机构订单索引
    #[pallet::storage]
    #[pallet::getter(fn institution_orders)]
    pub type InstitutionOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionIdLength>,                           // 主键：机构编码
        BoundedVec<BoundedVec<u8, T::MaxOrderCodeLength>, ConstU32<10000>>,  // 值：订单编码列表
        ValueQuery,                                                           // 查询策略：如果键不存在，返回空列表
    >;

    /// 待处理订单列表（按状态）
    #[pallet::storage]
    #[pallet::getter(fn orders_by_status)]
    pub type OrdersByStatus<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        OrderStatus,                                                          // 主键：订单状态
        BoundedVec<BoundedVec<u8, T::MaxOrderCodeLength>, ConstU32<10000>>,  // 值：订单编码列表
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 订单已创建 [订单编码, 创建者]
        OrderCreated(BoundedVec<u8, T::MaxOrderCodeLength>, T::AccountId),
        /// 订单状态已更新 [订单编码, 新状态(u8)]
        OrderStatusUpdated(BoundedVec<u8, T::MaxOrderCodeLength>, u8),
        /// 订单已取消 [订单编码]
        OrderCancelled(BoundedVec<u8, T::MaxOrderCodeLength>),
        /// 订单已删除 [订单编码]
        OrderDeleted(BoundedVec<u8, T::MaxOrderCodeLength>),
        /// 订单已完成 [订单编码]
        OrderCompleted(BoundedVec<u8, T::MaxOrderCodeLength>),
        /// 订单进入公证 [订单编码]
        OrderNotarizing(BoundedVec<u8, T::MaxOrderCodeLength>),
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
        /// 无效的订单状态转换
        InvalidStatusTransition,
        /// 用户订单列表已满
        UserOrderListFull,
        /// 机构订单列表已满
        InstitutionOrderListFull,
        /// 状态订单列表已满
        StatusOrderListFull,
        /// 无效的订单方向
        InvalidDirection,
        /// 无效的金额
        InvalidAmount,
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
            direction: u8,
            transaction_amount: u128,
            total_amount: u128,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_order_code = BoundedVec::<u8, T::MaxOrderCodeLength>::try_from(order_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查订单编码是否已存在
            ensure!(!Orders::<T>::contains_key(&bounded_order_code), Error::<T>::OrderCodeAlreadyExists);
            
            // 验证金额
            ensure!(transaction_amount > 0, Error::<T>::InvalidAmount);
            ensure!(total_amount >= transaction_amount, Error::<T>::InvalidAmount);
            
            // 转换订单方向
            let order_direction = match direction {
                0 => OrderDirection::UserSell,
                1 => OrderDirection::UserBuy,
                _ => return Err(Error::<T>::InvalidDirection.into()),
            };
            
            // 转换其他字段为边界向量
            let bounded_member_code = BoundedVec::<u8, T::MaxMemberCodeLength>::try_from(member_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionIdLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // 创建订单
            let order = Order {
                order_code: bounded_order_code.clone(),
                member_code: bounded_member_code.clone(),
                institution_code: bounded_institution_code.clone(),
                status: OrderStatus::Pending,
                direction: order_direction,
                created_time: current_block,
                updated_time: current_block,
                transaction_amount,
                total_amount,
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
            
            // 更新状态订单索引
            OrdersByStatus::<T>::try_mutate(OrderStatus::Pending, |orders| -> DispatchResult {
                orders.try_push(bounded_order_code.clone())
                    .map_err(|_| Error::<T>::StatusOrderListFull)?;
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
                
                let old_status = order.status.clone();
                
                // 将 u8 转换为 OrderStatus
                let new_status = match status {
                    0 => OrderStatus::Pending,
                    1 => OrderStatus::Paid,
                    2 => OrderStatus::Delivered,
                    3 => OrderStatus::Notarizing,
                    4 => OrderStatus::Cancelled,
                    5 => OrderStatus::Completed,
                    _ => return Err(Error::<T>::InvalidStatus.into()),
                };
                
                // 检查状态转换是否有效
                Self::validate_status_transition(&order.status, &new_status)?;
                
                // 更新状态和时间
                order.status = new_status.clone();
                order.updated_time = frame_system::Pallet::<T>::block_number();
                
                // 更新状态订单索引
                // 从旧状态列表中移除
                OrdersByStatus::<T>::mutate(&old_status, |orders| {
                    orders.retain(|code| code != &bounded_order_code);
                });
                
                // 添加到新状态列表
                OrdersByStatus::<T>::try_mutate(&new_status, |orders| -> DispatchResult {
                    orders.try_push(bounded_order_code.clone())
                        .map_err(|_| Error::<T>::StatusOrderListFull)?;
                    Ok(())
                })?;
                
                // 发出事件
                Self::deposit_event(Event::OrderStatusUpdated(bounded_order_code.clone(), status));
                
                // 如果是特殊状态，发出额外事件
                match new_status {
                    OrderStatus::Completed => {
                        Self::deposit_event(Event::OrderCompleted(bounded_order_code.clone()));
                    },
                    OrderStatus::Notarizing => {
                        Self::deposit_event(Event::OrderNotarizing(bounded_order_code.clone()));
                    },
                    _ => {}
                }
                
                Ok(())
            })
        }
        
        /// 取消订单
        #[pallet::call_index(2)]
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
                
                let old_status = order.status.clone();
                
                // 更新状态
                order.status = OrderStatus::Cancelled;
                order.updated_time = frame_system::Pallet::<T>::block_number();
                
                // 更新状态订单索引
                OrdersByStatus::<T>::mutate(&old_status, |orders| {
                    orders.retain(|code| code != &bounded_order_code);
                });
                
                OrdersByStatus::<T>::try_mutate(OrderStatus::Cancelled, |orders| -> DispatchResult {
                    orders.try_push(bounded_order_code.clone())
                        .map_err(|_| Error::<T>::StatusOrderListFull)?;
                    Ok(())
                })?;
                
                // 发出事件
                Self::deposit_event(Event::OrderCancelled(bounded_order_code.clone()));
                
                Ok(())
            })
        }
        
        /// 删除订单
        #[pallet::call_index(3)]
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
            
            // 从状态订单索引中移除
            OrdersByStatus::<T>::mutate(&order.status, |orders| {
                orders.retain(|code| code != &bounded_order_code);
            });
            
            // 删除订单
            Orders::<T>::remove(&bounded_order_code);
            
            // 发出事件
            Self::deposit_event(Event::OrderDeleted(bounded_order_code));
            
            Ok(())
        }
        
        /// 完成订单
        #[pallet::call_index(4)]
        #[pallet::weight(5_000)]
        pub fn complete_order(
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
                
                // 检查权限
                ensure!(order.creator == who, Error::<T>::NotAuthorized);
                
                // 只有已发货或公证中的订单可以完成
                ensure!(
                    matches!(order.status, OrderStatus::Delivered | OrderStatus::Notarizing),
                    Error::<T>::InvalidStatusTransition
                );
                
                let old_status = order.status.clone();
                
                // 更新状态
                order.status = OrderStatus::Completed;
                order.updated_time = frame_system::Pallet::<T>::block_number();
                
                // 更新状态订单索引
                OrdersByStatus::<T>::mutate(&old_status, |orders| {
                    orders.retain(|code| code != &bounded_order_code);
                });
                
                OrdersByStatus::<T>::try_mutate(OrderStatus::Completed, |orders| -> DispatchResult {
                    orders.try_push(bounded_order_code.clone())
                        .map_err(|_| Error::<T>::StatusOrderListFull)?;
                    Ok(())
                })?;
                
                // 发出事件
                Self::deposit_event(Event::OrderCompleted(bounded_order_code.clone()));
                
                Ok(())
            })
        }
    }
    
    // 辅助函数
    impl<T: Config> Pallet<T> {
        /// 验证状态转换是否有效
        fn validate_status_transition(from: &OrderStatus, to: &OrderStatus) -> DispatchResult {
            use OrderStatus::*;
            
            let valid_transition = match (from, to) {
                // 待支付可以转换为已支付、已取消或公证中
                (Pending, Paid) | (Pending, Cancelled) | (Pending, Notarizing) => true,
                // 已支付可以转换为已发货、已取消或公证中
                (Paid, Delivered) | (Paid, Cancelled) | (Paid, Notarizing) => true,
                // 已发货可以转换为已完成或公证中
                (Delivered, Completed) | (Delivered, Notarizing) => true,
                // 公证中可以转换为已完成或已取消
                (Notarizing, Completed) | (Notarizing, Cancelled) => true,
                // 其他转换无效
                _ => false,
            };
            
            ensure!(valid_transition, Error::<T>::InvalidStatusTransition);
            Ok(())
        }
    }
} 