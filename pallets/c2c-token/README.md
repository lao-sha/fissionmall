# C2C Token Pallet

C2C（Customer to Customer）Token 交易管理模块，支持用户之间的 Token 买卖交易。

## 功能特性

- 创建 Token（支持买入/卖出两种交易方向）
- 更新 Token 信息
- 更新 Token 状态（上架/下架）
- 更新 Token 价格
- 更新 Token 库存
- 删除 Token
- 交易 Token（根据交易方向自动处理买卖逻辑）

## 数据结构

### TokenStatus（Token 状态）

- `Available` (0) - 上架
- `Unavailable` (1) - 下架

### TokenTradableDirection（Token 交易方向）

- `Sell` (0) - 出售（卖家发布，买家购买）
- `Buy` (1) - 购买（买家发布求购，卖家出售）

### TokenInfo（Token 信息）

- `institution_code` - 所属机构代码
- `token_name` - Token 名称
- `category` - 分类
- `price` - Token 价格（u128）
- `token_trade_direction` - Token 交易方向
- `stock_quantity` - 库存数量
- `sales_quantity` - 销售数量
- `status` - Token 状态
- `creator` - 创建者账户

## 存储

### Tokens
使用双键存储映射（StorageDoubleMap）：
- 第一个键：Token 代码
- 第二个键：机构代码
- 值：Token 信息

### InstitutionTokens
机构 Token 索引：
- 键：机构代码
- 值：该机构的 Token 代码列表

### UserTokens
用户 Token 索引：
- 键：用户账户
- 值：(Token 代码, 机构代码) 元组列表

## 可调用函数

### create_token

创建新 Token。

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码
- `token_name` - Token 名称
- `category` - 分类
- `price` - 价格（必须大于0）
- `token_trade_direction` - 交易方向（0=出售，1=购买）
- `stock_quantity` - 初始库存

### update_token_info

更新 Token 信息。只有创建者可以更新。

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码
- `token_name` - Token 名称（可选）
- `category` - 分类（可选）
- `price` - 价格（可选）
- `token_trade_direction` - 交易方向（可选）

### update_token_status

更新 Token 状态（上架/下架）。只有创建者可以更新。

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码
- `status` - 新状态（0=上架，1=下架）

### update_token_price

更新 Token 价格。只有创建者可以更新。

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码
- `new_price` - 新价格（必须大于0）

### update_stock

更新 Token 库存。只有创建者可以更新。

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码
- `new_stock` - 新库存数量

### delete_token

删除 Token。只有创建者可以删除。

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码

### trade_token

交易 Token。根据 Token 的交易方向自动处理：
- 如果是 `Sell` 类型：买家购买，减少库存
- 如果是 `Buy` 类型：卖家出售，增加库存

参数：
- `token_code` - Token 代码
- `institution_code` - 机构代码
- `quantity` - 交易数量

## 事件

- `TokenCreated(Token代码, 机构代码, 创建者)` - Token已创建
- `TokenUpdated(Token代码, 机构代码)` - Token信息已更新
- `TokenStatusUpdated(Token代码, 机构代码, 新状态)` - Token状态已更新
- `TokenPriceUpdated(Token代码, 机构代码, 新价格)` - Token价格已更新
- `TokenStockUpdated(Token代码, 机构代码, 新库存)` - Token库存已更新
- `TokenDeleted(Token代码, 机构代码)` - Token已删除
- `TokenTraded(Token代码, 机构代码, 交易者, 数量)` - Token已交易

## 错误

- `TokenAlreadyExists` - Token已存在
- `TokenNotFound` - Token不存在
- `NotAuthorized` - 无权操作此Token
- `StringConversionError` - 字符串转换错误
- `InstitutionTokenListFull` - 机构Token列表已满
- `UserTokenListFull` - 用户Token列表已满
- `InsufficientStock` - 库存不足
- `InvalidStatus` - 无效的状态
- `InvalidPrice` - 无效的价格
- `InvalidTradeDirection` - 无效的交易方向

## 配置

在 runtime 中配置：

```rust
impl pallet_c2c_token::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxTokenCodeLength = ConstU32<64>;        // Token代码最大长度
    type MaxInstitutionCodeLength = ConstU32<64>;  // 机构代码最大长度
    type MaxNameLength = ConstU32<256>;            // 名称最大长度
    type MaxCategoryLength = ConstU32<128>;        // 分类最大长度
}
```

## 使用示例

```javascript
// 创建一个出售类型的 Token
const createSellToken = api.tx.c2cToken.createToken(
    'TOKEN-001',                             // Token代码
    'INST-001',                              // 机构代码
    '数字藏品 #001',                          // Token名称
    '数字艺术',                               // 分类
    100000,                                  // 价格（1000元，单位：分）
    0,                                       // 0 = Sell（出售）
    10                                       // 库存10个
);

// 创建一个求购类型的 Token
const createBuyToken = api.tx.c2cToken.createToken(
    'TOKEN-002',                             // Token代码
    'INST-001',                              // 机构代码
    '求购稀有数字藏品',                        // Token名称
    '数字艺术',                               // 分类
    50000,                                   // 价格（500元）
    1,                                       // 1 = Buy（求购）
    5                                        // 求购5个
);

// 更新价格
const updatePrice = api.tx.c2cToken.updateTokenPrice(
    'TOKEN-001',
    'INST-001',
    120000  // 新价格1200元
);

// 交易 Token
const trade = api.tx.c2cToken.tradeToken(
    'TOKEN-001',
    'INST-001',
    2  // 购买2个
);

// 查询 Token 信息
const tokenInfo = await api.query.c2cToken.tokens('TOKEN-001', 'INST-001');
console.log(tokenInfo.toHuman());

// 查询机构的所有 Token
const institutionTokens = await api.query.c2cToken.institutionTokens('INST-001');
console.log(institutionTokens.toHuman());

// 查询用户拥有的 Token
const userTokens = await api.query.c2cToken.userTokens(userAccount);
console.log(userTokens.toHuman());
```

## 交易逻辑说明

1. **Sell 类型 Token**：
   - 卖家创建 Token 并设置价格和库存
   - 买家调用 `trade_token` 购买，库存减少
   - 销售数量增加

2. **Buy 类型 Token**：
   - 买家创建求购 Token 并设置愿意支付的价格
   - 卖家调用 `trade_token` 出售，库存增加
   - 交易完成后，Token 会添加到交易者的列表中 