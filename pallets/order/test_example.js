// Example of how to interact with the Order pallet using Polkadot.js API

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { hexToU8a, stringToHex } = require('@polkadot/util');

async function main() {
    // Connect to the local node
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider: wsProvider });

    // Create a keyring instance
    const keyring = new Keyring({ type: 'sr25519' });
    
    // Add Alice account
    const alice = keyring.addFromUri('//Alice');

    // Example 1: Create an order
    console.log('Creating a new order...');
    
    const orderCode = 'ORDER-001';
    const memberCode = 'MEMBER-001';
    const institutionCode = 'INST-001';
    const freight = 50; // 50元运费
    
    // Contact information
    const phone = '+86 13800138000';
    const email = 'customer@example.com';
    const address = '北京市朝阳区某某街道1号';
    
    // Order items: (product_code, quantity, price_per_unit, weight)
    const items = [
        ['PROD-001', 2, 100, 500],  // 商品1: 2个，单价100元，单重500克
        ['PROD-002', 1, 200, 1000], // 商品2: 1个，单价200元，单重1000克
    ];
    
    const createOrderTx = api.tx.order.createOrder(
        orderCode,
        memberCode,
        institutionCode,
        freight,
        phone,
        email,
        address,
        items
    );
    
    // Send the transaction
    await createOrderTx.signAndSend(alice, ({ status, events }) => {
        if (status.isInBlock) {
            console.log(`Transaction included at blockHash ${status.asInBlock}`);
            
            // Process events
            events.forEach(({ event: { data, method, section } }) => {
                console.log(`\t${section}.${method}:`, data.toString());
            });
        }
    });
    
    // Example 2: Update order status (e.g., mark as paid)
    console.log('\nUpdating order status to Paid...');
    const updateStatusTx = api.tx.order.updateOrderStatus(orderCode, 1); // 1 = Paid
    
    await updateStatusTx.signAndSend(alice, ({ status }) => {
        if (status.isInBlock) {
            console.log('Order status updated successfully');
        }
    });
    
    // Example 3: Update express information
    console.log('\nUpdating express information...');
    const expressCompany = '顺丰快递';
    const expressNumber = 'SF1234567890';
    
    const updateExpressTx = api.tx.order.updateExpressInfo(
        orderCode,
        expressCompany,
        expressNumber
    );
    
    await updateExpressTx.signAndSend(alice, ({ status }) => {
        if (status.isInBlock) {
            console.log('Express information updated successfully');
        }
    });
    
    // Example 4: Query order information
    console.log('\nQuerying order information...');
    const order = await api.query.order.orders(orderCode);
    
    if (order.isSome) {
        const orderData = order.unwrap();
        console.log('Order details:', orderData.toHuman());
    }
    
    // Example 5: Query user orders
    console.log('\nQuerying user orders...');
    const userOrders = await api.query.order.userOrders(memberCode);
    console.log('User orders:', userOrders.toHuman());
    
    // Example 6: Query institution orders
    console.log('\nQuerying institution orders...');
    const institutionOrders = await api.query.order.institutionOrders(institutionCode);
    console.log('Institution orders:', institutionOrders.toHuman());
    
    // Disconnect
    await api.disconnect();
}

main().catch(console.error); 