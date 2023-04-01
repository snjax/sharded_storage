const {
    time,
    loadFixture,
  } = require("@nomicfoundation/hardhat-network-helpers");
  const { anyValue } = require("@nomicfoundation/hardhat-chai-matchers/withArgs");
  const { expect } = require("chai");

  describe("StateRegistry", function () {
    it("Should push a new state to the registry", async function () {
        const [owner, a1, a2, a3] = await ethers.getSigners();
        const StateRegistry = await ethers.getContractFactory("StateRegistry");
        const stateRegistry = await StateRegistry.deploy();

        await stateRegistry.connect(a1).pushState("0x1234");
        await stateRegistry.connect(a2).pushState("0x1234");
        await stateRegistry.connect(a3).pushState("0x1234");
        await stateRegistry.connect(a1).pushState("0x1234");

    });

  });