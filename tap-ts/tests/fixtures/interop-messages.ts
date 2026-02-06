/**
 * Test fixtures for interoperability testing between TAP and Veramo
 */

export const INTEROP_TEST_FIXTURES = {
  // Standard DIDComm v2 basic message (Veramo compatible)
  basicMessage: {
    plain: {
      id: '1234567890',
      type: 'https://didcomm.org/basicmessage/2.0/message',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      expires_time: 1516385931,
      body: {
        content: 'Hello, World!',
      },
    },
    // JWE format (encrypted DIDComm v2)
    encrypted: {
      protected: 'eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiYWxnIjoiRUNESC1FUyIsImVuYyI6IkEyNTZHQ00ifQ',
      recipients: [
        {
          header: {
            kid: 'did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2#z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc',
          },
          encrypted_key: 'EPYcG9pCKHHSYqOXxMzsAZAMJYnLr3pI4eAuQIlYnkiCwg8JKqV6nw',
        },
      ],
      iv: 'ESpmcSZt3aqTFSCb',
      ciphertext: 'TcSUIgqprMW_g7fwxPFOGpeni3d3bs2rm-p2h0dVLQchJPLeXwQ',
      tag: '2cLL79WTm6xnUJtQ8mLBkQ',
    },
  },

  // TAP Transfer message
  tapTransfer: {
    plain: {
      id: 'transfer-001',
      type: 'https://tap.rsvp/schema/1.0#Transfer',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      thid: 'thread-transfer-001',
      body: {
        amount: '100.00',
        asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
        originator: {
          '@id': 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
          metadata: {
            name: 'Alice Smith',
            accountNumber: '1234567890',
          },
        },
        beneficiary: {
          '@id': 'did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2',
          metadata: {
            name: 'Bob Jones',
            accountNumber: '0987654321',
          },
        },
        memo: 'Payment for services',
      },
    },
  },

  // TAP Payment message
  tapPayment: {
    plain: {
      id: 'payment-001',
      type: 'https://tap.rsvp/schema/1.0#Payment',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      body: {
        amount: '50.00',
        currency: 'USD',
        merchant: {
          '@id': 'did:web:merchant.example.com',
          metadata: {
            name: 'Example Merchant',
            category: 'retail',
          },
        },
        invoice: {
          invoiceNumber: 'INV-2024-001',
          items: [
            {
              description: 'Widget',
              quantity: 2,
              unitPrice: '25.00',
            },
          ],
        },
      },
    },
  },

  // DIDComm Trust Ping (Veramo compatible)
  trustPing: {
    plain: {
      id: 'ping-001',
      type: 'https://didcomm.org/trust-ping/2.0/ping',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      body: {
        response_requested: true,
      },
    },
  },

  // DIDComm Credential Offer (Veramo compatible)
  credentialOffer: {
    plain: {
      id: 'cred-offer-001',
      type: 'https://didcomm.org/issue-credential/3.0/offer-credential',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      body: {
        goal_code: 'issue-vc.credentialsubject',
        credential_preview: {
          type: 'https://didcomm.org/issue-credential/3.0/credential-preview',
          attributes: [
            {
              name: 'name',
              value: 'Alice Smith',
            },
            {
              name: 'degree',
              value: 'Bachelor of Science',
            },
            {
              name: 'date',
              value: '2024-01-15',
            },
          ],
        },
        formats: [
          {
            attach_id: 'offer-data',
            format: 'aries/ld-proof-vc-detail@v1.0',
          },
        ],
        'offers~attach': [
          {
            '@id': 'offer-data',
            'mime-type': 'application/json',
            data: {
              json: {
                credential: {
                  '@context': [
                    'https://www.w3.org/2018/credentials/v1',
                    'https://www.w3.org/2018/credentials/examples/v1',
                  ],
                  type: ['VerifiableCredential', 'UniversityDegreeCredential'],
                  credentialSubject: {
                    degree: {
                      type: 'BachelorDegree',
                      name: 'Bachelor of Science',
                    },
                  },
                },
                options: {
                  proofType: 'Ed25519Signature2018',
                },
              },
            },
          },
        ],
      },
    },
  },

  // TAP Connect message
  tapConnect: {
    plain: {
      id: 'connect-001',
      type: 'https://tap.rsvp/schema/1.0#Connect',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      body: {
        constraints: {
          asset_types: ['eip155:1/erc20:*'],
          currency_types: ['USD', 'EUR'],
          transaction_limits: {
            min_amount: '10.00',
            max_amount: '10000.00',
            daily_limit: '50000.00',
          },
        },
        metadata: {
          organization: 'Example Corp',
          relationship_type: 'business',
        },
      },
    },
  },

  // Mixed TAP and DIDComm headers
  hybridMessage: {
    plain: {
      id: 'hybrid-001',
      type: 'https://tap.rsvp/schema/1.0#Transfer',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      created_time: 1516269022,
      expires_time: 1516385931,
      thid: 'thread-123',
      pthid: 'parent-thread-456',
      body: {
        amount: '75.50',
        asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
        originator: {
          '@id': 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
        },
        beneficiary: {
          '@id': 'did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2',
        },
      },
      attachments: [
        {
          id: 'attachment-001',
          description: 'Transaction receipt',
          filename: 'receipt.pdf',
          media_type: 'application/pdf',
          format: 'base64',
          data: {
            base64: 'SGVsbG8gV29ybGQh',
          },
        },
      ],
    },
  },

  // Error test cases
  errorCases: {
    // Message with invalid type
    invalidType: {
      id: 'invalid-001',
      type: 'not-a-valid-type',
      from: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
      to: ['did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2'],
      body: {},
    },
    // Message missing required fields
    missingFields: {
      type: 'https://tap.rsvp/schema/1.0#Transfer',
      body: {
        amount: '100.00',
      },
    },
    // Malformed encrypted message
    malformedEncrypted: {
      protected: 'not-valid-base64',
      ciphertext: 'invalid',
    },
  },
};

/**
 * Expected results for compatibility tests
 */
export const EXPECTED_RESULTS = {
  // TAP should accept these DIDComm message types
  acceptedDIDCommTypes: [
    'https://didcomm.org/basicmessage/2.0/message',
    'https://didcomm.org/trust-ping/2.0/ping',
    'https://didcomm.org/trust-ping/2.0/ping-response',
    'https://didcomm.org/discover-features/2.0/queries',
    'https://didcomm.org/discover-features/2.0/disclose',
  ],

  // Veramo should accept these TAP message types (when properly formatted)
  acceptedTAPTypes: [
    'https://tap.rsvp/schema/1.0#Transfer',
    'https://tap.rsvp/schema/1.0#Payment',
    'https://tap.rsvp/schema/1.0#Authorize',
    'https://tap.rsvp/schema/1.0#Connect',
  ],

  // Encryption algorithms that should be supported
  supportedEncryption: {
    keyAgreement: ['ECDH-ES', 'ECDH-ES+A256KW', 'ECDH-1PU+A256KW'],
    contentEncryption: ['A256GCM', 'A256CBC-HS512', 'XC20P'],
  },

  // DID methods that should be resolvable
  supportedDIDMethods: ['did:key', 'did:web', 'did:ethr', 'did:ion'],
};

/**
 * Helper to create test keypairs for different DID methods
 */
export const TEST_KEYS = {
  ed25519: {
    privateKey: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    publicKey: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    did: 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK',
  },
  secp256k1: {
    privateKey: '0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321',
    publicKey: '0x0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba',
    did: 'did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme',
  },
  p256: {
    privateKey: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd',
    publicKey: '0xefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefab',
    did: 'did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169',
  },
};