// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

type TcbId is bytes32;

/// The quote did not link to the registration bundle.
error MismatchedRegistration(); // kPomqw== 90fa26ab
/// The registration has expired.
error RegistrationExpired(); // D+WbwA== 0fe59bc0
error InvalidQuote(); // +GGAMA== f8618030

contract AttestationToken is Ownable {
    struct Quote {
        bytes32 measurementHash;
        bytes32 userdata;
    }

    struct Registration {
        uint256 baseBlockNumber;
        bytes32 baseBlockHash;
        uint256 expiry;
        address registrant;
        uint256 tokenExpiry;
    }

    struct Attestation {
        uint256 expiry;
    }

    event Attested(address indexed requester, TcbId indexed tcbId, Quote quote);

    mapping(address => mapping(TcbId => Attestation)) public attestations;

    /// Mock attestation component.
    address private trustedSender_;

    constructor(address trustedSender) {
        trustedSender_ = trustedSender;
    }

    function attest(bytes calldata quoteData, Registration calldata reg) external returns (TcbId) {
        Quote memory quote = _parseQuote(quoteData);
        _validateRegistration(quote.userdata, reg);
        TcbId tcbId = _getTcbId(quote);
        attestations[reg.registrant][tcbId] = Attestation({expiry: reg.tokenExpiry});
        emit Attested(reg.registrant, tcbId, quote);
        return tcbId;
    }

    function getTcbId(bytes calldata quoteData) external view returns (TcbId) {
        Quote memory quote = _parseQuote(quoteData);
        return _getTcbId(quote);
    }

    function isAttested(address _whom, TcbId _tcbId) external view returns (bool) {
        return attestations[_whom][_tcbId].expiry > block.timestamp;
    }

    function setTrustedSender(address _whom) external onlyOwner {
        trustedSender_ = _whom;
    }

    function _getTcbId(Quote memory quote) internal view returns (TcbId) {
        return TcbId.wrap(keccak256(abi.encode(quote.measurementHash, "mock tcb", block.chainid)));
    }

    function _parseQuote(bytes calldata quote) internal view returns (Quote memory) {
        if (msg.sender != trustedSender_) revert InvalidQuote(); // mock verification
        return abi.decode(quote, (Quote));
    }

    function _validateRegistration(bytes32 expectedHash, Registration calldata reg) internal view {
        if (keccak256(abi.encode(reg)) != expectedHash) revert MismatchedRegistration();
        if (blockhash(reg.baseBlockNumber) != reg.baseBlockHash || block.timestamp >= reg.expiry) {
            revert RegistrationExpired();
        }
    }
}
