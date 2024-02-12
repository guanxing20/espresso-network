// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import { BN254 } from "bn254/BN254.sol";
import { LightClient as LC } from "../src/LightClient.sol";

contract DeployLightClientScript is Script {
    function run() external {
        string memory seedPhrase = vm.envString("MNEMONIC");
        uint256 privateKey = vm.deriveKey(seedPhrase, 0);
        vm.startBroadcast(privateKey);

        // For Decaf there will be only one epoch
        uint32 blocksPerEpoch = type(uint32).max;

        uint64 viewNum = 0;
        uint64 blockHeight = 0;
        BN254.ScalarField blockCommRoot = BN254.ScalarField.wrap(0);
        BN254.ScalarField feeLedgerComm = BN254.ScalarField.wrap(0);
        BN254.ScalarField stakeTableBlsKeyComm = BN254.ScalarField.wrap(
            21279528985922333501911071464328386841445872547720180015435547221260402565727
        );
        BN254.ScalarField stakeTableSchnorrKeyComm = BN254.ScalarField.wrap(
            7499525782104864821748149693309973670683978566054600186296438202262387687594
        );
        BN254.ScalarField stakeTableAmountComm = BN254.ScalarField.wrap(
            17674948780218579510668373650995978278726151659230619414891977565353113435659
        );
        uint256 threshold = 3;

        LC.LightClientState memory genesis = LC.LightClientState(
            viewNum,
            blockHeight,
            blockCommRoot,
            feeLedgerComm,
            stakeTableBlsKeyComm,
            stakeTableSchnorrKeyComm,
            stakeTableAmountComm,
            threshold
        );
        new LC(genesis, blocksPerEpoch);

        vm.stopBroadcast();
    }
}
