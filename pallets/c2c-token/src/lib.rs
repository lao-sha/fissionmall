#![cfg_attr(not(feature = "std"), no_std)]

/// C2C Token 交易管理模块
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
        
        /// Token代码最大长度
        #[pallet::constant]
        type MaxTokenCodeLength: Get<u32>;
        
        /// 机构代码最大长度
        #[pallet::constant]
        type MaxInstitutionCodeLength: Get<u32>;
        
        /// 名称最大长度
        #[pallet::constant]
        type MaxNameLength: Get<u32>;
        
        /// 分类最大长度
        #[pallet::constant]
        type MaxCategoryLength: Get<u32>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 状态枚举
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum TokenStatus {
        Available = 0,   // 上架
        Unavailable = 1, // 下架
    }

    /// token交易方向枚举
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[repr(u8)]
    pub enum TokenTradableDirection {
        Sell = 0, // 出售
        Buy = 1,  // 购买
    }

    /// token信息结构体
    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct TokenInfo<T: Config> {
        pub institution_code: BoundedVec<u8, T::MaxInstitutionCodeLength>, // 所属机构
        pub token_name: BoundedVec<u8, T::MaxNameLength>,                  // token名称
        pub category: BoundedVec<u8, T::MaxCategoryLength>,                // 分类
        pub price: u128,                                                   // token价格
        pub token_trade_direction: TokenTradableDirection,                 // token交易方向
        pub stock_quantity: u32,                                           // 库存数量
        pub sales_quantity: u32,                                           // 销售数量
        pub status: TokenStatus,                                           // token状态
        pub creator: T::AccountId,                                         // 创建者
    }

    /// 存储token信息的映射，主键为token ID
    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub type Tokens<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxTokenCodeLength>,       // token代码
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionCodeLength>, // token所属机构
        TokenInfo<T>,                                // token信息
        OptionQuery,                                 // 查询策略：如果键不存在，返回None
    >;

    /// 机构token索引
    #[pallet::storage]
    #[pallet::getter(fn institution_tokens)]
    pub type InstitutionTokens<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxInstitutionCodeLength>,                          // 机构代码
        BoundedVec<BoundedVec<u8, T::MaxTokenCodeLength>, ConstU32<10000>>,   // token代码列表
        ValueQuery,
    >;

    /// 用户拥有的token
    #[pallet::storage]
    #[pallet::getter(fn user_tokens)]
    pub type UserTokens<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,                                                          // 用户账户
        BoundedVec<(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>), ConstU32<1000>>, // (token代码, 机构代码)列表
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Token已创建 [token代码, 机构代码, 创建者]
        TokenCreated(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, T::AccountId),
        /// Token信息已更新 [token代码, 机构代码]
        TokenUpdated(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>),
        /// Token状态已更新 [token代码, 机构代码, 新状态(u8)]
        TokenStatusUpdated(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, u8),
        /// Token价格已更新 [token代码, 机构代码, 新价格]
        TokenPriceUpdated(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, u128),
        /// Token库存已更新 [token代码, 机构代码, 新库存]
        TokenStockUpdated(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, u32),
        /// Token已删除 [token代码, 机构代码]
        TokenDeleted(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>),
        /// Token已交易 [token代码, 机构代码, 买家/卖家, 数量]
        TokenTraded(BoundedVec<u8, T::MaxTokenCodeLength>, BoundedVec<u8, T::MaxInstitutionCodeLength>, T::AccountId, u32),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Token已存在
        TokenAlreadyExists,
        /// Token不存在
        TokenNotFound,
        /// 无权操作此Token
        NotAuthorized,
        /// 字符串转换错误
        StringConversionError,
        /// 机构Token列表已满
        InstitutionTokenListFull,
        /// 用户Token列表已满
        UserTokenListFull,
        /// 库存不足
        InsufficientStock,
        /// 无效的状态
        InvalidStatus,
        /// 无效的价格
        InvalidPrice,
        /// 无效的交易方向
        InvalidTradeDirection,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建新Token
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_token(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
            token_name: Vec<u8>,
            category: Vec<u8>,
            price: u128,
            token_trade_direction: u8,
            stock_quantity: u32,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 检查Token是否已存在
            ensure!(!Tokens::<T>::contains_key(&bounded_token_code, &bounded_institution_code), 
                Error::<T>::TokenAlreadyExists);
            
            // 验证价格
            ensure!(price > 0, Error::<T>::InvalidPrice);
            
            // 转换交易方向
            let trade_direction = match token_trade_direction {
                0 => TokenTradableDirection::Sell,
                1 => TokenTradableDirection::Buy,
                _ => return Err(Error::<T>::InvalidTradeDirection.into()),
            };
            
            // 转换其他字段
            let bounded_token_name = BoundedVec::<u8, T::MaxNameLength>::try_from(token_name)
                .map_err(|_| Error::<T>::StringConversionError)?;
                
            let bounded_category = BoundedVec::<u8, T::MaxCategoryLength>::try_from(category)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 创建Token信息
            let token_info = TokenInfo {
                institution_code: bounded_institution_code.clone(),
                token_name: bounded_token_name,
                category: bounded_category,
                price,
                token_trade_direction: trade_direction,
                stock_quantity,
                sales_quantity: 0,
                status: TokenStatus::Available,
                creator: who.clone(),
            };
            
            // 存储Token信息
            Tokens::<T>::insert(&bounded_token_code, &bounded_institution_code, &token_info);
            
            // 更新机构Token索引
            InstitutionTokens::<T>::try_mutate(&bounded_institution_code, |tokens| -> DispatchResult {
                tokens.try_push(bounded_token_code.clone())
                    .map_err(|_| Error::<T>::InstitutionTokenListFull)?;
                Ok(())
            })?;
            
            // 更新用户Token索引
            UserTokens::<T>::try_mutate(&who, |tokens| -> DispatchResult {
                tokens.try_push((bounded_token_code.clone(), bounded_institution_code.clone()))
                    .map_err(|_| Error::<T>::UserTokenListFull)?;
                Ok(())
            })?;
            
            // 发出事件
            Self::deposit_event(Event::TokenCreated(bounded_token_code, bounded_institution_code, who));
            
            Ok(())
        }
        
        /// 更新Token信息
        #[pallet::call_index(1)]
        #[pallet::weight(8_000)]
        pub fn update_token_info(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
            token_name: Option<Vec<u8>>,
            category: Option<Vec<u8>>,
            price: Option<u128>,
            token_trade_direction: Option<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新Token信息
            Tokens::<T>::try_mutate(&bounded_token_code, &bounded_institution_code, |maybe_token| -> DispatchResult {
                let token = maybe_token.as_mut().ok_or(Error::<T>::TokenNotFound)?;
                
                // 检查权限
                ensure!(token.creator == who, Error::<T>::NotAuthorized);
                
                // 更新各字段（如果提供）
                if let Some(name) = token_name {
                    token.token_name = BoundedVec::<u8, T::MaxNameLength>::try_from(name)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(cat) = category {
                    token.category = BoundedVec::<u8, T::MaxCategoryLength>::try_from(cat)
                        .map_err(|_| Error::<T>::StringConversionError)?;
                }
                
                if let Some(p) = price {
                    ensure!(p > 0, Error::<T>::InvalidPrice);
                    token.price = p;
                }
                
                if let Some(dir) = token_trade_direction {
                    token.token_trade_direction = match dir {
                        0 => TokenTradableDirection::Sell,
                        1 => TokenTradableDirection::Buy,
                        _ => return Err(Error::<T>::InvalidTradeDirection.into()),
                    };
                }
                
                // 发出事件
                Self::deposit_event(Event::TokenUpdated(bounded_token_code.clone(), bounded_institution_code.clone()));
                
                Ok(())
            })
        }
        
        /// 更新Token状态
        #[pallet::call_index(2)]
        #[pallet::weight(5_000)]
        pub fn update_token_status(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
            status: u8,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新Token状态
            Tokens::<T>::try_mutate(&bounded_token_code, &bounded_institution_code, |maybe_token| -> DispatchResult {
                let token = maybe_token.as_mut().ok_or(Error::<T>::TokenNotFound)?;
                
                // 检查权限
                ensure!(token.creator == who, Error::<T>::NotAuthorized);
                
                // 转换状态
                let new_status = match status {
                    0 => TokenStatus::Available,
                    1 => TokenStatus::Unavailable,
                    _ => return Err(Error::<T>::InvalidStatus.into()),
                };
                
                // 更新状态
                token.status = new_status;
                
                // 发出事件
                Self::deposit_event(Event::TokenStatusUpdated(
                    bounded_token_code.clone(), 
                    bounded_institution_code.clone(), 
                    status
                ));
                
                Ok(())
            })
        }
        
        /// 更新Token价格
        #[pallet::call_index(3)]
        #[pallet::weight(5_000)]
        pub fn update_token_price(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
            new_price: u128,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 验证价格
            ensure!(new_price > 0, Error::<T>::InvalidPrice);
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新Token价格
            Tokens::<T>::try_mutate(&bounded_token_code, &bounded_institution_code, |maybe_token| -> DispatchResult {
                let token = maybe_token.as_mut().ok_or(Error::<T>::TokenNotFound)?;
                
                // 检查权限
                ensure!(token.creator == who, Error::<T>::NotAuthorized);
                
                // 更新价格
                token.price = new_price;
                
                // 发出事件
                Self::deposit_event(Event::TokenPriceUpdated(
                    bounded_token_code.clone(), 
                    bounded_institution_code.clone(), 
                    new_price
                ));
                
                Ok(())
            })
        }
        
        /// 更新Token库存
        #[pallet::call_index(4)]
        #[pallet::weight(5_000)]
        pub fn update_stock(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
            new_stock: u32,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新Token库存
            Tokens::<T>::try_mutate(&bounded_token_code, &bounded_institution_code, |maybe_token| -> DispatchResult {
                let token = maybe_token.as_mut().ok_or(Error::<T>::TokenNotFound)?;
                
                // 检查权限
                ensure!(token.creator == who, Error::<T>::NotAuthorized);
                
                // 更新库存
                token.stock_quantity = new_stock;
                
                // 发出事件
                Self::deposit_event(Event::TokenStockUpdated(
                    bounded_token_code.clone(), 
                    bounded_institution_code.clone(), 
                    new_stock
                ));
                
                Ok(())
            })
        }
        
        /// 删除Token
        #[pallet::call_index(5)]
        #[pallet::weight(5_000)]
        pub fn delete_token(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取Token并检查权限
            let token = Tokens::<T>::get(&bounded_token_code, &bounded_institution_code)
                .ok_or(Error::<T>::TokenNotFound)?;
            
            ensure!(token.creator == who, Error::<T>::NotAuthorized);
            
            // 从机构Token索引中移除
            InstitutionTokens::<T>::mutate(&bounded_institution_code, |tokens| {
                tokens.retain(|code| code != &bounded_token_code);
            });
            
            // 从用户Token索引中移除
            UserTokens::<T>::mutate(&who, |tokens| {
                tokens.retain(|(code, inst)| code != &bounded_token_code || inst != &bounded_institution_code);
            });
            
            // 删除Token
            Tokens::<T>::remove(&bounded_token_code, &bounded_institution_code);
            
            // 发出事件
            Self::deposit_event(Event::TokenDeleted(bounded_token_code, bounded_institution_code));
            
            Ok(())
        }
        
        /// 交易Token（买入或卖出）
        #[pallet::call_index(6)]
        #[pallet::weight(8_000)]
        pub fn trade_token(
            origin: OriginFor<T>,
            token_code: Vec<u8>,
            institution_code: Vec<u8>,
            quantity: u32,
        ) -> DispatchResult {
            // 确认调用者身份
            let who = ensure_signed(origin)?;
            
            // 转换为边界向量
            let bounded_token_code = BoundedVec::<u8, T::MaxTokenCodeLength>::try_from(token_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            let bounded_institution_code = BoundedVec::<u8, T::MaxInstitutionCodeLength>::try_from(institution_code)
                .map_err(|_| Error::<T>::StringConversionError)?;
            
            // 获取并更新Token信息
            Tokens::<T>::try_mutate(&bounded_token_code, &bounded_institution_code, |maybe_token| -> DispatchResult {
                let token = maybe_token.as_mut().ok_or(Error::<T>::TokenNotFound)?;
                
                // 检查Token状态
                ensure!(token.status == TokenStatus::Available, Error::<T>::TokenNotFound);
                
                // 根据交易方向处理
                match token.token_trade_direction {
                    TokenTradableDirection::Sell => {
                        // 卖出模式：检查库存
                        ensure!(token.stock_quantity >= quantity, Error::<T>::InsufficientStock);
                        
                        // 更新库存和销售数量
                        token.stock_quantity = token.stock_quantity.saturating_sub(quantity);
                        token.sales_quantity = token.sales_quantity.saturating_add(quantity);
                    },
                    TokenTradableDirection::Buy => {
                        // 买入模式：增加库存
                        token.stock_quantity = token.stock_quantity.saturating_add(quantity);
                    }
                }
                
                // 发出事件
                Self::deposit_event(Event::TokenTraded(
                    bounded_token_code.clone(),
                    bounded_institution_code.clone(),
                    who.clone(),
                    quantity
                ));
                
                Ok(())
            })?;
            
            // 如果是买入模式，需要更新用户Token列表
            let token_info = Tokens::<T>::get(&bounded_token_code, &bounded_institution_code)
                .ok_or(Error::<T>::TokenNotFound)?;
                
            if token_info.token_trade_direction == TokenTradableDirection::Buy {
                // 添加到用户Token列表（如果还没有）
                UserTokens::<T>::try_mutate(&who, |tokens| -> DispatchResult {
                    let token_pair = (bounded_token_code.clone(), bounded_institution_code.clone());
                    if !tokens.contains(&token_pair) {
                        tokens.try_push(token_pair)
                            .map_err(|_| Error::<T>::UserTokenListFull)?;
                    }
                    Ok(())
                })?;
            }
            
            Ok(())
        }
    }
} 