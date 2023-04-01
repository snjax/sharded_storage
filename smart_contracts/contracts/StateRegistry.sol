// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract StateRegistry {
    mapping(address => bytes[]) public state;

    function pushState(bytes memory _state) public {
        state[msg.sender].push(_state);
    }

    function getStateHeight(address _address) public view returns (uint) {
        return state[_address].length;
    }

}