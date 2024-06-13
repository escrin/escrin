export const IIdentityRegistry = [
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "acceptRegistrationTransfer",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "permitter",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "pers",
        "type": "bytes"
      }
    ],
    "name": "createIdentity",
    "outputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "destroyIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "getPermitter",
    "outputs": [
      {
        "internalType": "contract IPermitter",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "getRegistrant",
    "outputs": [
      {
        "internalType": "address",
        "name": "current",
        "type": "address"
      },
      {
        "internalType": "address",
        "name": "proposed",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "to",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "name": "grantIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "proposeRegistrationTransfer",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "holder",
        "type": "address"
      },
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      }
    ],
    "name": "readPermit",
    "outputs": [
      {
        "components": [
          {
            "internalType": "uint64",
            "name": "expiry",
            "type": "uint64"
          }
        ],
        "internalType": "struct IIdentityRegistry.Permit",
        "name": "",
        "type": "tuple"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "from",
        "type": "address"
      }
    ],
    "name": "revokeIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "permitter",
        "type": "address"
      }
    ],
    "name": "setPermitter",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "IdentityCreated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "IdentityDestroyed",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "IdentityGranted",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "from",
        "type": "address"
      }
    ],
    "name": "IdentityRevoked",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      }
    ],
    "name": "PermitterChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "proposed",
        "type": "address"
      }
    ],
    "name": "RegistrationTransferProposed",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const IPermitter = [
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "possessor",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  }
] as const;

export const IdentityRegistry = [
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "acceptRegistrationTransfer",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "permitter",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "pers",
        "type": "bytes"
      }
    ],
    "name": "createIdentity",
    "outputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "destroyIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "getPermitter",
    "outputs": [
      {
        "internalType": "contract IPermitter",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "getRegistrant",
    "outputs": [
      {
        "internalType": "address",
        "name": "current",
        "type": "address"
      },
      {
        "internalType": "address",
        "name": "proposed",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "to",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "name": "grantIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "proposeRegistrationTransfer",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "holder",
        "type": "address"
      },
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "readPermit",
    "outputs": [
      {
        "components": [
          {
            "internalType": "uint64",
            "name": "expiry",
            "type": "uint64"
          }
        ],
        "internalType": "struct IIdentityRegistry.Permit",
        "name": "",
        "type": "tuple"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "from",
        "type": "address"
      }
    ],
    "name": "revokeIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "permitter",
        "type": "address"
      }
    ],
    "name": "setPermitter",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "IdentityCreated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "IdentityDestroyed",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "IdentityGranted",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "from",
        "type": "address"
      }
    ],
    "name": "IdentityRevoked",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      }
    ],
    "name": "PermitterChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "proposed",
        "type": "address"
      }
    ],
    "name": "RegistrationTransferProposed",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const OmniKeyStore = [
  {
    "inputs": [],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "acceptRegistrationTransfer",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "permitter",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "pers",
        "type": "bytes"
      }
    ],
    "name": "createIdentity",
    "outputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "destroyIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "eip712Domain",
    "outputs": [
      {
        "internalType": "bytes1",
        "name": "fields",
        "type": "bytes1"
      },
      {
        "internalType": "string",
        "name": "name",
        "type": "string"
      },
      {
        "internalType": "string",
        "name": "version",
        "type": "string"
      },
      {
        "internalType": "uint256",
        "name": "chainId",
        "type": "uint256"
      },
      {
        "internalType": "address",
        "name": "verifyingContract",
        "type": "address"
      },
      {
        "internalType": "bytes32",
        "name": "salt",
        "type": "bytes32"
      },
      {
        "internalType": "uint256[]",
        "name": "extensions",
        "type": "uint256[]"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "components": [
          {
            "components": [
              {
                "internalType": "IdentityId",
                "name": "identity",
                "type": "bytes32"
              },
              {
                "internalType": "address",
                "name": "requester",
                "type": "address"
              },
              {
                "internalType": "uint256",
                "name": "expiry",
                "type": "uint256"
              }
            ],
            "internalType": "struct OmniKeyStore.KeyRequest",
            "name": "req",
            "type": "tuple"
          },
          {
            "internalType": "bytes",
            "name": "sig",
            "type": "bytes"
          }
        ],
        "internalType": "struct OmniKeyStore.SignedKeyRequest",
        "name": "signedKeyReq",
        "type": "tuple"
      }
    ],
    "name": "getKey",
    "outputs": [
      {
        "internalType": "OmniKeyStore.Key",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "getPermitter",
    "outputs": [
      {
        "internalType": "contract IPermitter",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "getRegistrant",
    "outputs": [
      {
        "internalType": "address",
        "name": "current",
        "type": "address"
      },
      {
        "internalType": "address",
        "name": "proposed",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "components": [
          {
            "components": [
              {
                "internalType": "IdentityId",
                "name": "identity",
                "type": "bytes32"
              },
              {
                "internalType": "address",
                "name": "requester",
                "type": "address"
              },
              {
                "internalType": "uint256",
                "name": "expiry",
                "type": "uint256"
              }
            ],
            "internalType": "struct OmniKeyStore.KeyRequest",
            "name": "req",
            "type": "tuple"
          },
          {
            "internalType": "bytes",
            "name": "sig",
            "type": "bytes"
          }
        ],
        "internalType": "struct OmniKeyStore.SignedKeyRequest",
        "name": "signedKeyReq",
        "type": "tuple"
      }
    ],
    "name": "getSecondaryKey",
    "outputs": [
      {
        "internalType": "OmniKeyStore.Key",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "to",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "name": "grantIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "proposeRegistrationTransfer",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      },
      {
        "internalType": "bytes",
        "name": "pers",
        "type": "bytes"
      }
    ],
    "name": "provisionSecondaryKey",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "holder",
        "type": "address"
      },
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "readPermit",
    "outputs": [
      {
        "components": [
          {
            "internalType": "uint64",
            "name": "expiry",
            "type": "uint64"
          }
        ],
        "internalType": "struct IIdentityRegistry.Permit",
        "name": "",
        "type": "tuple"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "from",
        "type": "address"
      }
    ],
    "name": "revokeIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      }
    ],
    "name": "rotateKeys",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "permitter",
        "type": "address"
      }
    ],
    "name": "setPermitter",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [],
    "name": "EIP712DomainChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "IdentityCreated",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      }
    ],
    "name": "IdentityDestroyed",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "IdentityGranted",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "id",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "from",
        "type": "address"
      }
    ],
    "name": "IdentityRevoked",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      }
    ],
    "name": "PermitterChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "IdentityId",
        "name": "identityId",
        "type": "bytes32"
      },
      {
        "indexed": true,
        "internalType": "address",
        "name": "proposed",
        "type": "address"
      }
    ],
    "name": "RegistrationTransferProposed",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "ECDSAInvalidSignature",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "length",
        "type": "uint256"
      }
    ],
    "name": "ECDSAInvalidSignatureLength",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "s",
        "type": "bytes32"
      }
    ],
    "name": "ECDSAInvalidSignatureS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidShortString",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "KeyAlreadyProvisioned",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "KeyNotProvisioned",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "string",
        "name": "str",
        "type": "string"
      }
    ],
    "name": "StringTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const BaseNitroEnclavePermitter = [
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "burnt",
    "outputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "BindingMismatch",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertNotActive",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ContractExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DocAlreadyUsed",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidSignature",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const MultiNitroEnclavePermitter = [
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "upstream",
        "type": "address"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "burnt",
    "outputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "pcrs",
    "outputs": [
      {
        "internalType": "uint16",
        "name": "mask",
        "type": "uint16"
      },
      {
        "internalType": "bytes32",
        "name": "hash",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "components": [
          {
            "internalType": "uint16",
            "name": "mask",
            "type": "uint16"
          },
          {
            "internalType": "bytes32",
            "name": "hash",
            "type": "bytes32"
          }
        ],
        "internalType": "struct NE.PcrSelector",
        "name": "pcrSel",
        "type": "tuple"
      }
    ],
    "name": "setPCRs",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "BindingMismatch",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertNotActive",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ContractExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DocAlreadyUsed",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidSignature",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const NE = [
  {
    "inputs": [],
    "name": "ContractExpired",
    "type": "error"
  }
] as const;

export const Sig = [
  {
    "inputs": [],
    "name": "InvalidSignature",
    "type": "error"
  }
] as const;

export const StaticNitroEnclavePermitter = [
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "upstream",
        "type": "address"
      },
      {
        "internalType": "uint16",
        "name": "mask",
        "type": "uint16"
      },
      {
        "internalType": "bytes32",
        "name": "hash",
        "type": "bytes32"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "burnt",
    "outputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "pcrHash",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "pcrMask",
    "outputs": [
      {
        "internalType": "uint16",
        "name": "",
        "type": "uint16"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "BindingMismatch",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertNotActive",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ContractExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DocAlreadyUsed",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidSignature",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const X509 = [
  {
    "inputs": [],
    "name": "CertExpired",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "CertNotActive",
    "type": "error"
  }
] as const;

export const Permitter = [
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const ExperimentalSsssPermitter = [
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "upstream",
        "type": "address"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "approverRoots",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "eip712Domain",
    "outputs": [
      {
        "internalType": "bytes1",
        "name": "fields",
        "type": "bytes1"
      },
      {
        "internalType": "string",
        "name": "name",
        "type": "string"
      },
      {
        "internalType": "string",
        "name": "version",
        "type": "string"
      },
      {
        "internalType": "uint256",
        "name": "chainId",
        "type": "uint256"
      },
      {
        "internalType": "address",
        "name": "verifyingContract",
        "type": "address"
      },
      {
        "internalType": "bytes32",
        "name": "salt",
        "type": "bytes32"
      },
      {
        "internalType": "uint256[]",
        "name": "extensions",
        "type": "uint256[]"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "policyHashes",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "bytes32",
        "name": "approversRoot",
        "type": "bytes32"
      }
    ],
    "name": "setApproversRoot",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "bytes32",
        "name": "policyHash",
        "type": "bytes32"
      }
    ],
    "name": "setPolicyHash",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [],
    "name": "ApproverChange",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [],
    "name": "EIP712DomainChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [],
    "name": "PolicyChange",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ECDSAInvalidSignature",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "length",
        "type": "uint256"
      }
    ],
    "name": "ECDSAInvalidSignatureLength",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "s",
        "type": "bytes32"
      }
    ],
    "name": "ECDSAInvalidSignatureS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidShortString",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "MerkleProofInvalidMultiproof",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "string",
        "name": "str",
        "type": "string"
      }
    ],
    "name": "StringTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unsupported",
    "type": "error"
  }
] as const;

export const SsssPermitter = [
  {
    "inputs": [
      {
        "internalType": "address",
        "name": "upstream",
        "type": "address"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "constructor"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "approverRoots",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "eip712Domain",
    "outputs": [
      {
        "internalType": "bytes1",
        "name": "fields",
        "type": "bytes1"
      },
      {
        "internalType": "string",
        "name": "name",
        "type": "string"
      },
      {
        "internalType": "string",
        "name": "version",
        "type": "string"
      },
      {
        "internalType": "uint256",
        "name": "chainId",
        "type": "uint256"
      },
      {
        "internalType": "address",
        "name": "verifyingContract",
        "type": "address"
      },
      {
        "internalType": "bytes32",
        "name": "salt",
        "type": "bytes32"
      },
      {
        "internalType": "uint256[]",
        "name": "extensions",
        "type": "uint256[]"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "name": "policyHashes",
    "outputs": [
      {
        "internalType": "bytes32",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "bytes32",
        "name": "signersRoot",
        "type": "bytes32"
      },
      {
        "internalType": "uint256",
        "name": "threshold",
        "type": "uint256"
      }
    ],
    "name": "setApproversRoot",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "bytes32",
        "name": "policyHash",
        "type": "bytes32"
      }
    ],
    "name": "setPolicyHash",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      }
    ],
    "name": "ApproverChange",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [],
    "name": "EIP712DomainChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      }
    ],
    "name": "PolicyChange",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "ECDSAInvalidSignature",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "length",
        "type": "uint256"
      }
    ],
    "name": "ECDSAInvalidSignatureLength",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "s",
        "type": "bytes32"
      }
    ],
    "name": "ECDSAInvalidSignatureS",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidProof",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidShortString",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InvalidSsssSignature",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "MerkleProofInvalidMultiproof",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "NonceUsed",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "QuorumNotReached",
    "type": "error"
  },
  {
    "inputs": [
      {
        "internalType": "string",
        "name": "str",
        "type": "string"
      }
    ],
    "name": "StringTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const TrustedRelayerPermitter = [
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "uint64",
        "name": "duration",
        "type": "uint64"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "acquireIdentity",
    "outputs": [
      {
        "internalType": "uint64",
        "name": "expiry",
        "type": "uint64"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getTrustedRelayer",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "IdentityId",
        "name": "identity",
        "type": "bytes32"
      },
      {
        "internalType": "address",
        "name": "requester",
        "type": "address"
      },
      {
        "internalType": "bytes",
        "name": "context",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "authorization",
        "type": "bytes"
      }
    ],
    "name": "releaseIdentity",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "upstream",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "DurationTooLong",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const ITaskAcceptanceCriteria = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "uint256",
        "name": "taskId",
        "type": "uint256"
      }
    ],
    "name": "taskAcceptanceCriteria",
    "outputs": [
      {
        "internalType": "string",
        "name": "",
        "type": "string"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  }
] as const;

export const ITaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  }
] as const;

export const DelegatedTaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "sel",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "name": "UpstreamChanged",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "AcceptedTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SubmisionTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const StaticDelegatedTaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "sel",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getUpstreamTaskAcceptor",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "AcceptedTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SubmisionTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const TimelockedDelegatedTaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "sel",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "uint64",
        "name": "",
        "type": "uint64"
      }
    ],
    "name": "DelayChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "uint64",
        "name": "",
        "type": "uint64"
      }
    ],
    "name": "DelayIncoming",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "name": "UpstreamChanged",
    "type": "event"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "name": "UpstreamIncoming",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "AcceptedTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SubmisionTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const PermittedSubmitterTaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "sel",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getTrustedIdentity",
    "outputs": [
      {
        "internalType": "contract IIdentityRegistry",
        "name": "",
        "type": "address"
      },
      {
        "internalType": "IdentityId",
        "name": "",
        "type": "bytes32"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "AcceptedTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "InterfaceUnsupported",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SubmisionTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const TaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "sel",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "AcceptedTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SubmisionTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const TrustedSubmitterTaskAcceptor = [
  {
    "inputs": [
      {
        "internalType": "uint256[]",
        "name": "taskIds",
        "type": "uint256[]"
      },
      {
        "internalType": "bytes",
        "name": "proof",
        "type": "bytes"
      },
      {
        "internalType": "bytes",
        "name": "report",
        "type": "bytes"
      }
    ],
    "name": "acceptTaskResults",
    "outputs": [
      {
        "components": [
          {
            "internalType": "enum ITaskAcceptor.Quantifier",
            "name": "quantifier",
            "type": "uint8"
          },
          {
            "internalType": "uint256[]",
            "name": "taskIds",
            "type": "uint256[]"
          }
        ],
        "internalType": "struct ITaskAcceptor.TaskIdSelector",
        "name": "sel",
        "type": "tuple"
      }
    ],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "getTrustedSubmitter",
    "outputs": [
      {
        "internalType": "address",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "AcceptedTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "SubmisionTaskIdsNotSorted",
    "type": "error"
  },
  {
    "inputs": [],
    "name": "Unauthorized",
    "type": "error"
  }
] as const;

export const ITaskHub = [
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "context",
        "type": "bytes32"
      }
    ],
    "name": "notify",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "notify",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "address",
        "name": "generator",
        "type": "address"
      },
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "context",
        "type": "bytes32"
      }
    ],
    "name": "TasksAvailable",
    "type": "event"
  }
] as const;

export const TaskHub = [
  {
    "inputs": [
      {
        "internalType": "bytes32",
        "name": "context",
        "type": "bytes32"
      }
    ],
    "name": "notify",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "notify",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [
      {
        "internalType": "bytes4",
        "name": "interfaceId",
        "type": "bytes4"
      }
    ],
    "name": "supportsInterface",
    "outputs": [
      {
        "internalType": "bool",
        "name": "",
        "type": "bool"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": true,
        "internalType": "address",
        "name": "generator",
        "type": "address"
      },
      {
        "indexed": true,
        "internalType": "bytes32",
        "name": "context",
        "type": "bytes32"
      }
    ],
    "name": "TasksAvailable",
    "type": "event"
  }
] as const;

export const BaseTaskHubNotifier = [
  {
    "inputs": [],
    "name": "getTaskHub",
    "outputs": [
      {
        "internalType": "contract ITaskHub",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "TaskHubChanged",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "NotTaskHub",
    "type": "error"
  }
] as const;

export const TaskHubNotifier = [
  {
    "inputs": [],
    "name": "getTaskHub",
    "outputs": [
      {
        "internalType": "contract ITaskHub",
        "name": "",
        "type": "address"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "anonymous": false,
    "inputs": [
      {
        "indexed": false,
        "internalType": "address",
        "name": "to",
        "type": "address"
      }
    ],
    "name": "TaskHubChanged",
    "type": "event"
  },
  {
    "inputs": [],
    "name": "NotTaskHub",
    "type": "error"
  }
] as const;