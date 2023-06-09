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
    address private trustedSender;

    constructor(address _trustedSender) {
        trustedSender = _trustedSender;
    }

    function attest(bytes calldata _quote, Registration calldata _reg) external returns (TcbId) {
        Quote memory quote = _parseQuote(_quote);
        _validateRegistration(quote.userdata, _reg);
        TcbId tcbId = _getTcbId(quote);
        attestations[_reg.registrant][tcbId] = Attestation({expiry: _reg.tokenExpiry});
        emit Attested(_reg.registrant, tcbId, quote);
        return tcbId;
    }

    function getTcbId(bytes calldata _quote) external view returns (TcbId) {
        Quote memory quote = _parseQuote(_quote);
        return _getTcbId(quote);
    }

    function isAttested(address _whom, TcbId _tcbId) external view returns (bool) {
        return attestations[_whom][_tcbId].expiry > block.timestamp;
    }

    function setTrustedSender(address _whom) external onlyOwner {
        trustedSender = _whom;
    }

    function _getTcbId(Quote memory quote) internal view returns (TcbId) {
        return TcbId.wrap(keccak256(abi.encode(quote.measurementHash, "mock tcb", block.chainid)));
    }

    function _parseQuote(bytes calldata _quote) internal view returns (Quote memory quote) {
        quote = abi.decode(_quote, (Quote));
        if (msg.sender != trustedSender) revert InvalidQuote(); // mock verification
    }

    function _validateRegistration(
        bytes32 _expectedHash,
        Registration calldata _reg
    ) internal view {
        if (keccak256(abi.encode(_reg)) != _expectedHash) revert MismatchedRegistration();
        if (blockhash(_reg.baseBlockNumber) != _reg.baseBlockHash || block.timestamp >= _reg.expiry)
            revert RegistrationExpired();
    }
}
