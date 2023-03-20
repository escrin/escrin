// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

// import "hardhat/console.sol";

type AttestationTokenId is bytes32;

/// The quote did not link to the registration data.
error MismatchedRegistration();
/// The challenge has expired.
error ChallengeExpired();

contract AttestationToken {
    struct Quote {
        bytes32 measurementHash;
        bytes32 userdata;
        bytes otherStuff;
    }

    mapping(address => mapping(AttestationTokenId => bool)) public attestations;

    bytes32 public challenge;
    uint256 public challengeExpiry;

    function attest(
        Quote calldata _quote,
        bytes calldata _registration
    ) external returns (AttestationTokenId) {
        if (block.timestamp > challengeExpiry) revert ChallengeExpired();
        if (_quote.userdata != keccak256(bytes.concat(challenge, _registration)))
            revert MismatchedRegistration();
        // TODO: verify issuance
        (address registrant) = abi.decode(_registration, (address));
        AttestationTokenId attid = AttestationTokenId.wrap(_quote.measurementHash);
        attestations[registrant][attid] = true;
        return attid;
    }

    function refreshChallenge() external {
        challenge = keccak256(abi.encode(4)); // verified random
        challengeExpiry = type(uint256).max;
    }

    function isAttested(address _whom, AttestationTokenId _attid) external view returns (bool) {
        return attestations[_whom][_attid];
    }
}
