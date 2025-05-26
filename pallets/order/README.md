# Order Pallet

订单管理模块，用于管理电商系统中的订单。

## 功能特性

- 创建订单
- 更新订单状态
- 更新快递信息
- 取消订单
- 删除订单

## 数据结构

### OrderStatus（订单状态）

- `Pending` - 待支付
- `Paid` - 已支付
- `Delivered` - 已发货
- `Cancelled` - 已取消
- `Completed` - 已完成
- `Refunded` - 已退款

### ContactInformation（联系信息）

- `phone` - 电话号码（可选，最大32字节）
- `email` - 邮箱（可选，最大128字节）
- `address` - 地址（可选，最大512字节）

### OrderItem（订单商品项）

- `product_code` - 商品编码（最大64字节）
- `quantity` - 商品数量
- `price_per_unit` - 单价
- `weight` - 商品重量

### Order（订单）

- `order_code` - 订单编码
- `member_code` - 会员编码
- `institution_code` - 机构编码
- `status` - 订单状态
- `created_time` - 创建时间
- `updated_time` - 更新时间
- `total_amount` - 总金额
- `total_weight` - 总重量
- `freight` - 运费
- `contact_information` - 联系信息
- `items` - 订单商品列表
- `express_company` - 快递公司名称
- `express_number` - 快递单号
- `creator` - 创建者账户

## 存储

- `Orders` - 订单存储映射，key为订单编码
- `UserOrders` - 用户订单索引，key为用户编码，value为订单编码列表
- `InstitutionOrders` - 机构订单索引，key为机构编码，value为订单编码列表

## 可调用函数

### create_order

创建新订单。

参数：
- `order_code` - 订单编码
- `member_code` - 会员编码
- `institution_code` - 机构编码
- `freight` - 运费
- `phone` - 电话号码（可选）
- `email` - 邮箱（可选）
- `address` - 地址（可选）
- `items` - 订单商品列表：Vec<(商品编码, 数量, 单价, 重量)>

### update_order_status

更新订单状态。只有订单创建者可以更新。

参数：
- `order_code` - 订单编码
- `status` - 新状态（0-5对应不同状态）

状态转换规则：
- 待支付 → 已支付/已取消
- 已支付 → 已发货/已退款/已取消
- 已发货 → 已完成
- 已完成 → 已退款

### update_express_info

更新订单快递信息。只有订单创建者可以更新。

参数：
- `order_code` - 订单编码
- `express_company` - 快递公司名称
- `express_number` - 快递单号

### cancel_order

取消订单。只有订单创建者可以取消，且订单必须处于待支付或已支付状态。

参数：
- `order_code` - 订单编码

### delete_order

删除订单。只有订单创建者可以删除。

参数：
- `order_code` - 订单编码

## 事件

- `OrderCreated(订单编码, 创建者)` - 订单已创建
- `OrderStatusUpdated(订单编码, 新状态)` - 订单状态已更新
- `OrderExpressInfoUpdated(订单编码)` - 订单快递信息已更新
- `OrderCancelled(订单编码)` - 订单已取消
- `OrderDeleted(订单编码)` - 订单已删除

## 错误

- `OrderCodeAlreadyExists` - 订单编码已存在
- `OrderNotFound` - 订单不存在
- `NotAuthorized` - 无权操作此订单
- `StringConversionError` - 字符串转换错误
- `InvalidStatus` - 无效状态
- `EmptyOrderItems` - 订单项为空
- `TooManyOrderItems` - 订单项数量超过限制
- `InvalidStatusTransition` - 无效的订单状态转换
- `UserOrderListFull` - 用户订单列表已满
- `InstitutionOrderListFull` - 机构订单列表已满

## 配置

在runtime中配置：

```rust
impl pallet_order::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxOrderCodeLength = ConstU32<64>;      // 订单编码最大长度
    type MaxMemberCodeLength = ConstU32<64>;     // 会员编码最大长度
    type MaxInstitutionIdLength = ConstU32<64>;  // 机构ID最大长度
    type MaxOrderItems = ConstU32<100>;          // 订单项最大数量
    type MaxExpressCompanyLength = ConstU32<128>;// 快递公司名称最大长度
    type MaxExpressNumberLength = ConstU32<64>;  // 快递单号最大长度
}
``` 