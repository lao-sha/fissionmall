# Product Pallet

商品管理模块，用于管理电商系统中的商品信息。

## 功能特性

- 创建商品
- 更新商品信息
- 更新商品状态（上架/下架）
- 更新商品库存
- 删除商品
- 购买商品（减少库存，增加销售量）

## 数据结构

### ProductStatus（商品状态）

- `Available` (0) - 上架
- `Unavailable` (1) - 下架

### ProductInfo（商品信息）

- `product_name` - 商品名称
- `category` - 分类
- `brand` - 商品品牌
- `authorized_member_groups` - 授权用户组列表
- `original_price` - 原价
- `current_price` - 现价
- `description` - 商品描述
- `main_image` - 主图 URL
- `detail_images` - 详情图 URL 列表
- `stock_quantity` - 库存数量
- `sales_quantity` - 销售数量
- `weight` - 重量（克）
- `status` - 商品状态
- `profit_ratio` - 分润比例
- `created_date` - 创建日期
- `creator` - 创建者账户

## 存储

### Products
使用双键存储映射（StorageDoubleMap）：
- 第一个键：商品代码
- 第二个键：机构代码
- 值：商品信息

### InstitutionProducts
机构商品索引：
- 键：机构代码
- 值：该机构的商品代码列表

## 可调用函数

### create_product

创建新商品。

参数：
- `product_code` - 商品代码
- `institution_code` - 机构代码
- `product_name` - 商品名称
- `category` - 分类
- `brand` - 品牌
- `authorized_member_groups` - 授权用户组列表
- `original_price` - 原价
- `current_price` - 现价
- `description` - 描述
- `main_image` - 主图 URL
- `detail_images` - 详情图 URL 列表
- `stock_quantity` - 初始库存
- `weight` - 重量
- `profit_ratio` - 分润比例

### update_product_info

更新商品信息。只有创建者可以更新。

参数：
- `product_code` - 商品代码
- `institution_code` - 机构代码
- 其他字段为可选参数

### update_product_status

更新商品状态（上架/下架）。只有创建者可以更新。

参数：
- `product_code` - 商品代码
- `institution_code` - 机构代码
- `status` - 新状态（0=上架，1=下架）

### update_stock

更新商品库存。只有创建者可以更新。

参数：
- `product_code` - 商品代码
- `institution_code` - 机构代码
- `new_stock` - 新库存数量

### delete_product

删除商品。只有创建者可以删除。

参数：
- `product_code` - 商品代码
- `institution_code` - 机构代码

### purchase_product

购买商品，自动减少库存并增加销售量。

参数：
- `product_code` - 商品代码
- `institution_code` - 机构代码
- `quantity` - 购买数量

## 事件

- `ProductCreated(商品代码, 机构代码, 创建者)` - 商品已创建
- `ProductUpdated(商品代码, 机构代码)` - 商品信息已更新
- `ProductStatusUpdated(商品代码, 机构代码, 新状态)` - 商品状态已更新
- `ProductStockUpdated(商品代码, 机构代码, 新库存)` - 商品库存已更新
- `ProductDeleted(商品代码, 机构代码)` - 商品已删除

## 错误

- `ProductAlreadyExists` - 商品已存在
- `ProductNotFound` - 商品不存在
- `NotAuthorized` - 无权操作此商品
- `StringConversionError` - 字符串转换错误
- `TooManyAuthorizedGroups` - 授权用户组数量超过限制
- `TooManyDetailImages` - 详情图数量超过限制
- `InvalidPrice` - 无效的价格（现价高于原价）
- `InvalidProfitRatio` - 无效的分润比例
- `InstitutionProductListFull` - 机构商品列表已满
- `InsufficientStock` - 库存不足
- `InvalidStatus` - 无效的状态值

## 配置

在 runtime 中配置：

```rust
impl pallet_product::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxProductCodeLength = ConstU32<64>;          // 商品代码最大长度
    type MaxInstitutionCodeLength = ConstU32<64>;      // 机构代码最大长度
    type MaxNameLength = ConstU32<256>;                // 名称最大长度
    type MaxCategoryLength = ConstU32<128>;            // 分类最大长度
    type MaxBrandLength = ConstU32<128>;               // 品牌最大长度
    type MaxAuthorizedMemberGroup = ConstU32<64>;      // 授权用户组名称最大长度
    type MaxAuthorizedGroups = ConstU32<10>;           // 授权用户组最大数量
    type MaxDescriptionLength = ConstU32<1024>;        // 描述最大长度
    type MaxImageUrlLength = ConstU32<512>;            // 图片URL最大长度
    type MaxDetailImages = ConstU32<10>;               // 详情图最大数量
}
```

## 使用示例

```javascript
// 创建商品
const createProduct = api.tx.product.createProduct(
    'PROD-001',                              // 商品代码
    'INST-001',                              // 机构代码
    '高端智能手机',                           // 商品名称
    '电子产品',                               // 分类
    'Apple',                                 // 品牌
    [['VIP会员'], ['普通会员']],              // 授权用户组
    999900,                                  // 原价（9999元，单位：分）
    899900,                                  // 现价（8999元）
    '最新款智能手机，搭载A17处理器',           // 描述
    'https://example.com/main.jpg',          // 主图
    [                                        // 详情图
        'https://example.com/detail1.jpg',
        'https://example.com/detail2.jpg'
    ],
    100,                                     // 库存
    200,                                     // 重量（200克）
    Perbill.from_percent(10)                 // 10%分润
);

// 更新商品状态为下架
const updateStatus = api.tx.product.updateProductStatus(
    'PROD-001',
    'INST-001',
    1  // 1 = 下架
);

// 购买商品
const purchase = api.tx.product.purchaseProduct(
    'PROD-001',
    'INST-001',
    2  // 购买2个
);
``` 