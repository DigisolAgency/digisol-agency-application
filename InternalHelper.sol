// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import './interfaces/ILendingAggregator.sol';

contract InternalHelper {
    address internal constant V1_POOL = 0x398eC7346DcD622eDc5ae82352F02bE94C62d119;
    address internal constant V1_CORE = 0x3dfd23A6c5E8BbcFc9581d2E864a68feb6a076d3;

    address internal constant V2_POOL = 0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9; // 0x4bd5643ac6f66a5237E18bfA7d47cF22f1c9F210;
    address internal constant V2_DATA_PROVIDER = 0x057835Ad21a177dbdd3090bB1CAE03EaCF78Fc6d; // 0x927F584d4321C1dCcBf5e2902368124b02419a1E;
    address internal constant V2_ORACLE = 0xA50ba011c48153De246E5192C8f9258A2ba79Ca9;

    address internal constant V3_POOL = 0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2; // 0x7b5C526B7F8dfdff278b4a3e045083FBA4028790;
    address internal constant V3_DATA_PROVIDER = 0x7B4EB56E7CD4b454BA8ff71E4518426369a138a3; // 0xa41E284482F9923E265832bE59627d91432da76C;
    address internal constant V3_ORACLE = 0x54586bE62E3c3580375aE3723C145253060Ca0C2; // 0x9F616c65b5298E24e155E4486e114516BC635b63;

    address internal constant COMOPUND_CONTROLLER = 0x3d9819210A31b4961b30EF54bE2aeD79B9c9Cd3B; // 0x05Df6C772A563FfB37fD3E04C1A279Fb30228621;
    address internal constant COMOPUND_ORACLE = 0x50ce56A3239671Ab62f185704Caedf626352741e;

    address internal constant CHAINLINK_ORACLE = 0x76B47460d7F7c5222cFb6b6A75615ab10895DDe4; // 0x2cb0d5755436ED904D7D0fbBACc6176286c55667;

    uint256 internal constant LIQUIDATION_PROTOCOL_FEE_MASK =  0xFFFFFFFFFFFFFFFFFFFFFF0000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF; // prettier-ignore

    struct UserReserve {
        address asset;
        uint256 collateralBalance;
        uint256 borrowBalance;
    }

    struct UserData {
        address addr;
        uint256 totalCollateralETH;
        uint256 totalBorrowsETH;
        uint256 healthFactor;
    }

    struct UserDataWithReserves {
        UserData userData;
        UserReserve[] reserves;
    }

    struct AAVEReservesData {
        address asset;
        uint256 price;
        uint256 decimals;
        uint256 bonus;
    }

    struct AAVEV3ReservesData {
        address asset;
        uint256 ethPrice;
        uint256 nativePrice;
        uint256 decimals;
        uint256 bonus;
        uint256 fee;
    }

    struct CompoundReservesData {
        address market;
        address underlying;
        uint256 ethPrice;
        uint256 nativePrice;
        uint256 decimals;
        uint256 bonus;
        uint256 collateralFactor;
    }

    function _aaveV1GetUserData(
        address _user
    ) internal view returns (uint256 totalCollateralETH, uint256 totalBorrowsETH, uint256 healthFactor) {
        (bool success, bytes memory data) = V1_POOL.staticcall(
            abi.encodeWithSignature('getUserAccountData(address)', _user)
        );

        if (success) {
            (, totalCollateralETH, totalBorrowsETH, , , , , healthFactor) = abi.decode(
                data,
                (uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint256)
            );
        }
    }

    function _aaveV2GetUserData(
        address _user
    ) internal view returns (uint256 totalCollateralETH, uint256 totalDebtETH, uint256 healthFactor) {
        (bool success, bytes memory data) = V2_POOL.staticcall(
            abi.encodeWithSignature('getUserAccountData(address)', _user)
        );

        if (success) {
            (totalCollateralETH, totalDebtETH, , , , healthFactor) = abi.decode(
                data,
                (uint256, uint256, uint256, uint256, uint256, uint256)
            );
        }
    }

    function _aaveV3GetUserData(
        address _user,
        uint256 _ethPrice
    ) internal view returns (uint256 totalCollateralBase, uint256 totalDebtBase, uint256 healthFactor) {
        (bool success, bytes memory data) = V3_POOL.staticcall(
            abi.encodeWithSignature('getUserAccountData(address)', _user)
        );

        if (success) {
            (totalCollateralBase, totalDebtBase, , , , healthFactor) = abi.decode(
                data,
                (uint256, uint256, uint256, uint256, uint256, uint256)
            );

            totalCollateralBase = (totalCollateralBase * 10 ** 18) / _ethPrice;
            totalDebtBase = (totalDebtBase * 10 ** 18) / _ethPrice;
        }
    }

    function _getCompoundUserData(
        UserReserve[] memory userReserves
    ) internal view returns (uint256 healthFactor, uint256 totalBorrowsETH, uint256 totalCollateralETH) {
        uint256 totalCollateralFactor;
        uint256 length = userReserves.length;

        uint256 ethPrice = ICompoundOracle(COMOPUND_ORACLE).getUnderlyingPrice(
            0x4Ddc2D193948926D02f9B1fE9e1daa0718270ED5
        );

        for (uint256 i; i < length; i++) {
            uint256 collateralBalance = userReserves[i].collateralBalance;
            uint256 borrowBalance = userReserves[i].borrowBalance;
            address market = userReserves[i].asset;

            uint256 collateralInETH;
            uint256 debtInETH;
            if (market != 0x4Ddc2D193948926D02f9B1fE9e1daa0718270ED5) {
                uint256 price = ICompoundOracle(COMOPUND_ORACLE).getUnderlyingPrice(market);

                collateralInETH = (collateralBalance * price) / ethPrice;
                debtInETH = (borrowBalance * price) / ethPrice;
            } else {
                collateralInETH = collateralBalance;
                debtInETH = borrowBalance;
            }

            ICompoundController.Market memory marketData = ICompoundController(COMOPUND_CONTROLLER).markets(market);
            uint256 collateralFactorInETH = marketData.collateralFactorMantissa * collateralInETH;

            totalBorrowsETH += debtInETH;
            totalCollateralETH += collateralInETH;
            totalCollateralFactor += collateralFactorInETH;
        }

        if (totalBorrowsETH != 0) {
            healthFactor = totalCollateralFactor / totalBorrowsETH;
        }
    }
}
