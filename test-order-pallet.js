const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
  // 等待加密库准备就绪
  await cryptoWaitReady();

  // 连接到本地节点
  const provider = new WsProvider('ws://localhost:9944');
  const api = await ApiPromise.create({ provider });

  // 创建密钥对
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  console.log('已连接到节点，链名称:', await api.rpc.system.chain());
  
  // 检查 Order pallet 是否存在
  if (api.tx.order) {
    console.log('✅ Order pallet 已成功添加到 runtime!');
    
    // 显示 Order pallet 的所有可用方法
    console.log('\nOrder pallet 可用的交易方法:');
    Object.keys(api.tx.order).forEach(method => {
      console.log(`  - ${method}`);
    });
    
    // 显示 Order pallet 的存储项
    console.log('\nOrder pallet 的存储项:');
    Object.keys(api.query.order).forEach(storage => {
      console.log(`  - ${storage}`);
    });
    
    // 创建一个测试订单
    console.log('\n准备创建测试订单...');
    
    const orderCode = 'TEST_ORDER_001';
    const memberCode = 'MEMBER_001';
    const institutionCode = 'INST_001';
    const freight = 10;
    const phone = '13800138000';
    const email = 'test@example.com';
    const address = '测试地址123号';
    const items = [
      ['PRODUCT_001', 2, 100, 500], // [产品代码, 数量, 单价, 重量]
      ['PRODUCT_002', 1, 200, 300]
    ];
    
    // 创建订单交易
    const createTx = api.tx.order.createOrder(
      orderCode,
      memberCode,
      institutionCode,
      freight,
      phone,
      email,
      address,
      items
    );
    
    // 发送交易
    const hash = await createTx.signAndSend(alice, (result) => {
      console.log(`交易状态: ${result.status}`);
      
      if (result.status.isInBlock) {
        console.log(`✅ 交易已包含在区块中: ${result.status.asInBlock}`);
      } else if (result.status.isFinalized) {
        console.log(`✅ 交易已最终确认: ${result.status.asFinalized}`);
        
        // 检查事件
        result.events.forEach(({ event }) => {
          if (event.section === 'order') {
            console.log(`📢 Order pallet 事件: ${event.method}`);
            console.log(`   数据:`, event.data.toString());
          }
        });
      }
    });
    
    console.log(`交易哈希: ${hash}`);
    
  } else {
    console.log('❌ Order pallet 未找到!');
  }
  
  // 等待几秒后断开连接
  setTimeout(() => {
    api.disconnect();
    console.log('\n已断开连接');
  }, 10000);
}

main().catch(console.error); 