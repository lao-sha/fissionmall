# C2C Order Pallet

C2C（Customer to Customer）订单管理模块，用于管理用户之间的交易订单。

## 功能特性

- 创建订单（支持用户买入/卖出两种方向）
- 更新订单状态
- 取消订单
- 删除订单
- 完成订单
- 订单状态索引管理
- 支持公证流程

## 数据结构

### OrderStatus（订单状态）

- `Pending` (0) - 待支付
- `Paid` (1) - 已支付
- `Delivered` (2) - 已发货
- `Notarizing` (3) - 公证中
- `Cancelled` (4) - 已取消
- `Completed` (5) - 已完成

### OrderDirection（订单方向）

- `UserSell` (0) - 用户出售
- `UserBuy` (1) - 用户购买

### Order（订单信息）

- `order_code` - 订单编码
- `member_code` - 用户编码
- `institution_code` - 所属机构编码
- `status` - 订单状态
- `direction` - 订单方向
- `created_time` - 创建时间
- `updated_time` - 更新时间
- `transaction_amount` - 交易金额（u128）
- `total_amount` - 总金额（u128）
- `creator` - 创建者账户

## 存储

### Orders
订单存储映射：
- 键：订单编码
- 值：订单信息

### UserOrders
用户订单索引：
- 键：用户编码
- 值：该用户的订单编码列表

### InstitutionOrders
机构订单索引：
- 键：机构编码
- 值：该机构的订单编码列表

### OrdersByStatus
按状态分类的订单索引：
- 键：订单状态
- 值：处于该状态的订单编码列表

## 可调用函数

### create_order

创建新订单。

参数：
- `order_code` - 订单编码
- `member_code` - 会员编码
- `institution_code` - 机构编码
- `direction` - 订单方向（0=用户出售，1=用户购买）
- `transaction_amount` - 交易金额（必须大于0）
- `total_amount` - 总金额（必须大于等于交易金额）

### update_order_status

更新订单状态。只有订单创建者可以更新。

参数：
- `order_code` - 订单编码
- `status` - 新状态（0-5对应不同状态）

状态转换规则：
- 待支付 → 已支付/已取消/公证中
- 已支付 → 已发货/已取消/公证中
- 已发货 → 已完成/公证中
- 公证中 → 已完成/已取消

### cancel_order

取消订单。只有订单创建者可以取消，且订单必须处于待支付或已支付状态。

参数：
- `order_code` - 订单编码

### delete_order

删除订单。只有订单创建者可以删除。

参数：
- `order_code` - 订单编码

### complete_order

完成订单。只有订单创建者可以完成，且订单必须处于已发货或公证中状态。

参数：
- `order_code` - 订单编码

## 事件

- `OrderCreated(订单编码, 创建者)` - 订单已创建
- `OrderStatusUpdated(订单编码, 新状态)` - 订单状态已更新
- `OrderCancelled(订单编码)` - 订单已取消
- `OrderDeleted(订单编码)` - 订单已删除
- `OrderCompleted(订单编码)` - 订单已完成
- `OrderNotarizing(订单编码)` - 订单进入公证

## 错误

- `OrderCodeAlreadyExists` - 订单编码已存在
- `OrderNotFound` - 订单不存在
- `NotAuthorized` - 无权操作此订单
- `StringConversionError` - 字符串转换错误
- `InvalidStatus` - 无效状态
- `InvalidStatusTransition` - 无效的订单状态转换
- `UserOrderListFull` - 用户订单列表已满
- `InstitutionOrderListFull` - 机构订单列表已满
- `StatusOrderListFull` - 状态订单列表已满
- `InvalidDirection` - 无效的订单方向
- `InvalidAmount` - 无效的金额

## 配置

在 runtime 中配置：

```rust
impl pallet_c2c_order::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxOrderCodeLength = ConstU32<64>;      // 订单编码最大长度
    type MaxMemberCodeLength = ConstU32<64>;     // 会员编码最大长度
    type MaxInstitutionIdLength = ConstU32<64>;  // 机构ID最大长度
}
```

## 使用示例

```javascript
// 创建一个用户出售订单
const createSellOrder = api.tx.c2cOrder.createOrder(
    'ORDER-001',                             // 订单编码
    'MEMBER-001',                            // 会员编码
    'INST-001',                              // 机构编码
    0,                                       // 0 = UserSell（用户出售）
    500000,                                  // 交易金额（5000元，单位：分）
    550000                                   // 总金额（5500元，含手续费）
);

// 创建一个用户购买订单
const createBuyOrder = api.tx.c2cOrder.createOrder(
    'ORDER-002',                             // 订单编码
    'MEMBER-002',                            // 会员编码
    'INST-001',                              // 机构编码
    1,                                       // 1 = UserBuy（用户购买）
    300000,                                  // 交易金额（3000元）
    300000                                   // 总金额（3000元）
);

// 更新订单状态为已支付
const updateToPaid = api.tx.c2cOrder.updateOrderStatus(
    'ORDER-001',
    1  // 1 = Paid
);

// 更新订单状态为公证中
const updateToNotarizing = api.tx.c2cOrder.updateOrderStatus(
    'ORDER-001',
    3  // 3 = Notarizing
);

// 完成订单
const completeOrder = api.tx.c2cOrder.completeOrder('ORDER-001');

// 取消订单
const cancelOrder = api.tx.c2cOrder.cancelOrder('ORDER-002');

// 查询订单信息
const orderInfo = await api.query.c2cOrder.orders('ORDER-001');
console.log(orderInfo.toHuman());

// 查询用户的所有订单
const userOrders = await api.query.c2cOrder.userOrders('MEMBER-001');
console.log(userOrders.toHuman());

// 查询特定状态的订单
const pendingOrders = await api.query.c2cOrder.ordersByStatus(0); // 0 = Pending
console.log(pendingOrders.toHuman());
```

## 订单流程说明

### 正常交易流程：
1. **Pending（待支付）** → **Paid（已支付）** → **Delivered（已发货）** → **Completed（已完成）**

### 需要公证的流程：
1. **Pending/Paid/Delivered** → **Notarizing（公证中）** → **Completed（已完成）**

### 取消流程：
1. **Pending/Paid** → **Cancelled（已取消）**
2. **Notarizing** → **Cancelled（已取消）**

## 特别说明

1. **订单方向**：
   - `UserSell`：用户作为卖家，出售商品或服务
   - `UserBuy`：用户作为买家，购买商品或服务

2. **金额说明**：
   - `transaction_amount`：实际交易金额
   - `total_amount`：总金额（包含手续费、运费等）

3. **公证流程**：
   - 当交易出现争议时，可以进入公证流程
   - 公证中的订单可以最终完成或被取消

4. **状态索引**：
   - 系统自动维护按状态分类的订单索引
   - 便于快速查询特定状态的所有订单 