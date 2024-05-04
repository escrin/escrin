export const IIdentityRegistry = [
  {
    "type": "function",
    "name": "acceptRegistrationTransfer",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "createIdentity",
    "inputs": [
      {
        "name": "permitter",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "pers",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "destroyIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getPermitter",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IPermitter"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getRegistrant",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "current",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "proposed",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "grantIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "proposeRegistrationTransfer",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "readPermit",
    "inputs": [
      {
        "name": "holder",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct IIdentityRegistry.Permit",
        "components": [
          {
            "name": "expiry",
            "type": "uint64",
            "internalType": "uint64"
          }
        ]
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "revokeIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "from",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setPermitter",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "permitter",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "IdentityCreated",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": false,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityDestroyed",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityGranted",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityRevoked",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "from",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "PermitterChanged",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "RegistrationTransferProposed",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "proposed",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const IPermitter = [
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "possessor",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  }
] as const;

export const IdentityRegistry = [
  {
    "type": "function",
    "name": "acceptRegistrationTransfer",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "createIdentity",
    "inputs": [
      {
        "name": "permitter",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "pers",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "destroyIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getPermitter",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IPermitter"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getRegistrant",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "current",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "proposed",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "grantIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "proposeRegistrationTransfer",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "readPermit",
    "inputs": [
      {
        "name": "holder",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct IIdentityRegistry.Permit",
        "components": [
          {
            "name": "expiry",
            "type": "uint64",
            "internalType": "uint64"
          }
        ]
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "revokeIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "from",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setPermitter",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "permitter",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "IdentityCreated",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": false,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityDestroyed",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityGranted",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityRevoked",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "from",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "PermitterChanged",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "RegistrationTransferProposed",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "proposed",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const OmniKeyStore = [
  {
    "type": "constructor",
    "inputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "acceptRegistrationTransfer",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "createIdentity",
    "inputs": [
      {
        "name": "permitter",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "pers",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "destroyIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "eip712Domain",
    "inputs": [],
    "outputs": [
      {
        "name": "fields",
        "type": "bytes1",
        "internalType": "bytes1"
      },
      {
        "name": "name",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "version",
        "type": "string",
        "internalType": "string"
      },
      {
        "name": "chainId",
        "type": "uint256",
        "internalType": "uint256"
      },
      {
        "name": "verifyingContract",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "salt",
        "type": "bytes32",
        "internalType": "bytes32"
      },
      {
        "name": "extensions",
        "type": "uint256[]",
        "internalType": "uint256[]"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getKey",
    "inputs": [
      {
        "name": "signedKeyReq",
        "type": "tuple",
        "internalType": "struct OmniKeyStore.SignedKeyRequest",
        "components": [
          {
            "name": "req",
            "type": "tuple",
            "internalType": "struct OmniKeyStore.KeyRequest",
            "components": [
              {
                "name": "identity",
                "type": "bytes32",
                "internalType": "IdentityId"
              },
              {
                "name": "requester",
                "type": "address",
                "internalType": "address"
              },
              {
                "name": "expiry",
                "type": "uint256",
                "internalType": "uint256"
              }
            ]
          },
          {
            "name": "sig",
            "type": "bytes",
            "internalType": "bytes"
          }
        ]
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "OmniKeyStore.Key"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getPermitter",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IPermitter"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getRegistrant",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "current",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "proposed",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "getSecondaryKey",
    "inputs": [
      {
        "name": "signedKeyReq",
        "type": "tuple",
        "internalType": "struct OmniKeyStore.SignedKeyRequest",
        "components": [
          {
            "name": "req",
            "type": "tuple",
            "internalType": "struct OmniKeyStore.KeyRequest",
            "components": [
              {
                "name": "identity",
                "type": "bytes32",
                "internalType": "IdentityId"
              },
              {
                "name": "requester",
                "type": "address",
                "internalType": "address"
              },
              {
                "name": "expiry",
                "type": "uint256",
                "internalType": "uint256"
              }
            ]
          },
          {
            "name": "sig",
            "type": "bytes",
            "internalType": "bytes"
          }
        ]
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "OmniKeyStore.Key"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "grantIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "proposeRegistrationTransfer",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "provisionSecondaryKey",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "pers",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "readPermit",
    "inputs": [
      {
        "name": "holder",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct IIdentityRegistry.Permit",
        "components": [
          {
            "name": "expiry",
            "type": "uint64",
            "internalType": "uint64"
          }
        ]
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "revokeIdentity",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "from",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "rotateKeys",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setPermitter",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "permitter",
        "type": "address",
        "internalType": "address"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "EIP712DomainChanged",
    "inputs": [],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityCreated",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": false,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityDestroyed",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityGranted",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "to",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "IdentityRevoked",
    "inputs": [
      {
        "name": "id",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "from",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "PermitterChanged",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "RegistrationTransferProposed",
    "inputs": [
      {
        "name": "identityId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "IdentityId"
      },
      {
        "name": "proposed",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "ECDSAInvalidSignature",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ECDSAInvalidSignatureLength",
    "inputs": [
      {
        "name": "length",
        "type": "uint256",
        "internalType": "uint256"
      }
    ]
  },
  {
    "type": "error",
    "name": "ECDSAInvalidSignatureS",
    "inputs": [
      {
        "name": "s",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ]
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidShortString",
    "inputs": []
  },
  {
    "type": "error",
    "name": "KeyAlreadyProvisioned",
    "inputs": []
  },
  {
    "type": "error",
    "name": "KeyNotProvisioned",
    "inputs": []
  },
  {
    "type": "error",
    "name": "StringTooLong",
    "inputs": [
      {
        "name": "str",
        "type": "string",
        "internalType": "string"
      }
    ]
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const BaseNitroEnclavePermitter = [
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "burnt",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "BindingMismatch",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertNotActive",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ContractExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DocAlreadyUsed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DurationTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidSignature",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const MultiNitroEnclavePermitter = [
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "upstream",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "burnt",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "pcrs",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "mask",
        "type": "uint16",
        "internalType": "uint16"
      },
      {
        "name": "hash",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setPCRs",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "pcrSel",
        "type": "tuple",
        "internalType": "struct NE.PcrSelector",
        "components": [
          {
            "name": "mask",
            "type": "uint16",
            "internalType": "uint16"
          },
          {
            "name": "hash",
            "type": "bytes32",
            "internalType": "bytes32"
          }
        ]
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "BindingMismatch",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertNotActive",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ContractExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DocAlreadyUsed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DurationTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidSignature",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const NE = [
  {
    "type": "error",
    "name": "ContractExpired",
    "inputs": []
  }
] as const;

export const Sig = [
  {
    "type": "error",
    "name": "InvalidSignature",
    "inputs": []
  }
] as const;

export const StaticNitroEnclavePermitter = [
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "upstream",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "mask",
        "type": "uint16",
        "internalType": "uint16"
      },
      {
        "name": "hash",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "burnt",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "pcrHash",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "pcrMask",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "uint16",
        "internalType": "uint16"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "BindingMismatch",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertNotActive",
    "inputs": []
  },
  {
    "type": "error",
    "name": "ContractExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DocAlreadyUsed",
    "inputs": []
  },
  {
    "type": "error",
    "name": "DurationTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InvalidSignature",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const X509 = [
  {
    "type": "error",
    "name": "CertExpired",
    "inputs": []
  },
  {
    "type": "error",
    "name": "CertNotActive",
    "inputs": []
  }
] as const;

export const Permitter = [
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "DurationTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const ExperimentalSsssPermitter = [
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "upstream",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "approverRoots",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "policyHashes",
    "inputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setApproversRoot",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "approversRoot",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "setPolicyHash",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "policyHash",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "ApproverChange",
    "inputs": [],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "PolicyChange",
    "inputs": [],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "DurationTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unsupported",
    "inputs": []
  }
] as const;

export const TrustedRelayerPermitter = [
  {
    "type": "function",
    "name": "acquireIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "duration",
        "type": "uint64",
        "internalType": "uint64"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "expiry",
        "type": "uint64",
        "internalType": "uint64"
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getTrustedRelayer",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "releaseIdentity",
    "inputs": [
      {
        "name": "identity",
        "type": "bytes32",
        "internalType": "IdentityId"
      },
      {
        "name": "requester",
        "type": "address",
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "authorization",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "upstream",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "DurationTooLong",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const ITaskAcceptanceCriteria = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "taskAcceptanceCriteria",
    "inputs": [
      {
        "name": "taskId",
        "type": "uint256",
        "internalType": "uint256"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "string",
        "internalType": "string"
      }
    ],
    "stateMutability": "view"
  }
] as const;

export const ITaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  }
] as const;

export const DelegatedTaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "sel",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "UpstreamChanged",
    "inputs": [
      {
        "name": "",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "AcceptedTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SubmisionTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const StaticDelegatedTaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "sel",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getUpstreamTaskAcceptor",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "AcceptedTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SubmisionTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const TimelockedDelegatedTaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "sel",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "DelayChanged",
    "inputs": [
      {
        "name": "",
        "type": "uint64",
        "indexed": false,
        "internalType": "uint64"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "DelayIncoming",
    "inputs": [
      {
        "name": "",
        "type": "uint64",
        "indexed": false,
        "internalType": "uint64"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "UpstreamChanged",
    "inputs": [
      {
        "name": "",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "UpstreamIncoming",
    "inputs": [
      {
        "name": "",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "AcceptedTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SubmisionTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const PermittedSubmitterTaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "sel",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getTrustedIdentity",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract IIdentityRegistry"
      },
      {
        "name": "",
        "type": "bytes32",
        "internalType": "IdentityId"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "AcceptedTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "InterfaceUnsupported",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SubmisionTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const TaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "sel",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "AcceptedTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SubmisionTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const TrustedSubmitterTaskAcceptor = [
  {
    "type": "function",
    "name": "acceptTaskResults",
    "inputs": [
      {
        "name": "taskIds",
        "type": "uint256[]",
        "internalType": "uint256[]"
      },
      {
        "name": "proof",
        "type": "bytes",
        "internalType": "bytes"
      },
      {
        "name": "report",
        "type": "bytes",
        "internalType": "bytes"
      }
    ],
    "outputs": [
      {
        "name": "sel",
        "type": "tuple",
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "components": [
          {
            "name": "quantifier",
            "type": "uint8",
            "internalType": "enum ITaskAcceptor.Quantifier"
          },
          {
            "name": "taskIds",
            "type": "uint256[]",
            "internalType": "uint256[]"
          }
        ]
      }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "getTrustedSubmitter",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "address"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "error",
    "name": "AcceptedTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "SubmisionTaskIdsNotSorted",
    "inputs": []
  },
  {
    "type": "error",
    "name": "Unauthorized",
    "inputs": []
  }
] as const;

export const ITaskHub = [
  {
    "type": "function",
    "name": "notify",
    "inputs": [
      {
        "name": "context",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "notify",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "TasksAvailable",
    "inputs": [
      {
        "name": "generator",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      }
    ],
    "anonymous": false
  }
] as const;

export const TaskHub = [
  {
    "type": "function",
    "name": "notify",
    "inputs": [
      {
        "name": "context",
        "type": "bytes32",
        "internalType": "bytes32"
      }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "notify",
    "inputs": [],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "supportsInterface",
    "inputs": [
      {
        "name": "interfaceId",
        "type": "bytes4",
        "internalType": "bytes4"
      }
    ],
    "outputs": [
      {
        "name": "",
        "type": "bool",
        "internalType": "bool"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "TasksAvailable",
    "inputs": [
      {
        "name": "generator",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      },
      {
        "name": "context",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      }
    ],
    "anonymous": false
  }
] as const;

export const BaseTaskHubNotifier = [
  {
    "type": "function",
    "name": "getTaskHub",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract ITaskHub"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "TaskHubChanged",
    "inputs": [
      {
        "name": "to",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "NotTaskHub",
    "inputs": []
  }
] as const;

export const TaskHubNotifier = [
  {
    "type": "function",
    "name": "getTaskHub",
    "inputs": [],
    "outputs": [
      {
        "name": "",
        "type": "address",
        "internalType": "contract ITaskHub"
      }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "TaskHubChanged",
    "inputs": [
      {
        "name": "to",
        "type": "address",
        "indexed": false,
        "internalType": "address"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "NotTaskHub",
    "inputs": []
  }
] as const;