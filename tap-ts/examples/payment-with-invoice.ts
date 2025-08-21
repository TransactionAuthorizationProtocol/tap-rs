/**
 * Payment with Invoice Example
 * 
 * This example demonstrates a payment request with detailed invoice
 */

import { TapAgent } from '@taprsvp/agent';

async function main() {
  console.log('TAP Payment with Invoice Example\n');
  
  // Create merchant agent
  console.log('Creating merchant agent...');
  const merchant = await TapAgent.create({ keyType: 'Ed25519' });
  console.log('Merchant DID:', merchant.did);
  
  // Create customer agent
  console.log('\nCreating customer agent...');
  const customer = await TapAgent.create({ keyType: 'Ed25519' });
  console.log('Customer DID:', customer.did);
  
  // Merchant creates payment request with invoice
  console.log('\n--- Step 1: Merchant creates payment request ---');
  const paymentRequest = await merchant.createMessage('Payment', {
    amount: '2,745.99',
    currency: 'USD',
    merchant: {
      '@id': merchant.did,
      '@type': 'https://schema.org/Organization',
      name: 'TechSupplies Inc.',
      mcc: '5734',  // Computer software stores
      url: 'https://techsupplies.example.com',
      email: 'orders@techsupplies.example.com',
      telephone: '+1-555-0123',
      leiCode: '969500KN90DZLPGW6898',
      countryOfRegistration: 'US'
    },
    settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7',
    invoice: {
      invoiceNumber: 'INV-2024-001234',
      invoiceDate: new Date().toISOString(),
      dueDate: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(), // 30 days
      customerReference: 'PO-98765',
      items: [
        {
          description: 'Dell Latitude 5540 Laptop',
          quantity: 2,
          unitPrice: '1,199.99',
          totalPrice: '2,399.98',
          sku: 'DELL-LAT-5540'
        },
        {
          description: 'USB-C Docking Station',
          quantity: 2,
          unitPrice: '89.99',
          totalPrice: '179.98',
          sku: 'DOCK-USBC-01'
        },
        {
          description: 'Wireless Mouse',
          quantity: 2,
          unitPrice: '29.99',
          totalPrice: '59.98',
          sku: 'MOUSE-WL-02'
        },
        {
          description: 'Laptop Bag',
          quantity: 2,
          unitPrice: '49.99',
          totalPrice: '99.98',
          sku: 'BAG-LAP-15'
        }
      ],
      subtotal: '2,739.92',
      tax: {
        rate: '8.625%',
        amount: '236.31'
      },
      shipping: '29.99',
      discount: {
        description: 'First-time customer discount',
        amount: '260.23'
      },
      total: '2,745.99',
      paymentTerms: 'Net 30',
      notes: 'Thank you for your business! Please reference invoice number with payment.'
    },
    memo: 'Q4 2024 Equipment Purchase',
    agents: []  // Could include payment processors, escrow agents, etc.
  });
  
  paymentRequest.to = [customer.did];
  
  const packedPayment = await merchant.pack(paymentRequest);
  console.log('Payment request created');
  console.log('Invoice number:', paymentRequest.body.invoice.invoiceNumber);
  console.log('Total amount:', paymentRequest.body.amount);
  console.log('Items:', paymentRequest.body.invoice.items.length);
  
  // Customer receives payment request
  console.log('\n--- Step 2: Customer receives payment request ---');
  const receivedPayment = await customer.unpack(packedPayment.message);
  console.log('Payment request from:', receivedPayment.body.merchant.metadata.name);
  console.log('\nInvoice Details:');
  console.log('- Invoice #:', receivedPayment.body.invoice.invoiceNumber);
  console.log('- Due Date:', receivedPayment.body.invoice.dueDate);
  console.log('- Items:');
  receivedPayment.body.invoice.items.forEach(item => {
    console.log(`  * ${item.quantity}x ${item.description} @ ${item.unitPrice} = ${item.totalPrice}`);
  });
  console.log('- Subtotal:', receivedPayment.body.invoice.subtotal);
  console.log('- Tax:', receivedPayment.body.invoice.tax.amount);
  console.log('- Shipping:', receivedPayment.body.invoice.shipping);
  console.log('- Discount:', receivedPayment.body.invoice.discount.amount);
  console.log('- TOTAL:', receivedPayment.body.invoice.total);
  
  // Customer authorizes payment
  console.log('\n--- Step 3: Customer authorizes payment ---');
  const authorize = await customer.createMessage('Authorize', {
    transaction_id: receivedPayment.id,
    settlement_address: '0x1234567890abcdef1234567890abcdef12345678',
    expiry: new Date(Date.now() + 1 * 60 * 60 * 1000).toISOString() // 1 hour
  }, {
    thid: receivedPayment.id,
    to: [merchant.did]
  });
  
  const packedAuth = await customer.pack(authorize);
  console.log('Payment authorized');
  console.log('Customer settlement address:', authorize.body.settlement_address);
  
  // Merchant receives authorization
  console.log('\n--- Step 4: Merchant receives authorization ---');
  const receivedAuth = await merchant.unpack(packedAuth.message);
  console.log('Authorization received for invoice:', receivedPayment.body.invoice.invoiceNumber);
  
  // Merchant confirms settlement
  console.log('\n--- Step 5: Merchant confirms settlement ---');
  const settle = await merchant.createMessage('Settle', {
    transaction_id: receivedPayment.id,
    settlement_id: `eip155:1:0x${Math.random().toString(16).slice(2, 42)}`,
    amount: receivedPayment.body.amount
  }, {
    thid: receivedPayment.id,
    to: [customer.did]
  });
  
  const packedSettle = await merchant.pack(settle);
  console.log('Settlement confirmed');
  console.log('Transaction hash:', settle.body.settlement_id);
  
  // Customer receives settlement confirmation
  console.log('\n--- Step 6: Customer receives settlement ---');
  const receivedSettle = await customer.unpack(packedSettle.message);
  console.log('Payment complete!');
  console.log('Invoice', receivedPayment.body.invoice.invoiceNumber, 'has been paid');
  console.log('Amount:', receivedSettle.body.amount);
  console.log('Settlement ID:', receivedSettle.body.settlement_id);
  
  console.log('\n✅ Payment with invoice completed successfully!');
  
  // Alternative: Customer rejects payment
  console.log('\n--- Alternative Flow: Rejection ---');
  const reject = await customer.createMessage('Reject', {
    transaction_id: receivedPayment.id,
    reason: 'Items not matching purchase order specifications'
  }, {
    thid: receivedPayment.id,
    to: [merchant.did]
  });
  
  const packedReject = await customer.pack(reject);
  const receivedReject = await merchant.unpack(packedReject.message);
  console.log('Payment rejected. Reason:', receivedReject.body.reason);
  
  // Clean up
  merchant.dispose();
  customer.dispose();
}

// Run the example
main().catch(error => {
  console.error('Error:', error);
  process.exit(1);
});