// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import './InternalHelper.sol';

contract LendingAggregator is InternalHelper {
    function aaveV1GetGlobalReservesData() external view returns (AAVEReservesData[] memory reservesData) {
        ILendingPoolV1 pool = ILendingPoolV1(V1_POOL);
        ILendingPoolV1Core core = ILendingPoolV1Core(V1_CORE);

        IOracle oracle = IOracle(CHAINLINK_ORACLE);

        address[] memory assets = pool.getReserves();

        uint256 length = assets.length;

        reservesData = new AAVEReservesData[](assets.length);

        for (uint256 i; i < length; i++) {
            address asset = assets[i];

            reservesData[i] = AAVEReservesData({
                asset: asset,
                price: oracle.getAssetPrice(asset),
                decimals: core.getReserveDecimals(asset),
                bonus: core.getReserveLiquidationBonus(asset)
            });
        }

        return reservesData;
    }

    function aaveV1GetUsersData(address[] calldata users) external view returns (UserData[] memory usersData) {
        uint256 size = users.length;
        usersData = new UserData[](size);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            (uint256 totalCollateralETH, uint256 totalBorrowsETH, uint256 healthFactor) = _aaveV1GetUserData(user);

            usersData[i] = UserData({
                addr: user,
                totalCollateralETH: totalCollateralETH,
                totalBorrowsETH: totalBorrowsETH,
                healthFactor: healthFactor
            });
        }

        return usersData;
    }

    function aaveV1GetUsersDataWithReserves(
        address[] calldata users
    ) external view returns (UserDataWithReserves[] memory usersDataWithReserves) {
        ILendingPoolV1 pool = ILendingPoolV1(V1_POOL);
        ILendingPoolV1Core core = ILendingPoolV1Core(V1_CORE);

        address[] memory assets = pool.getReserves();

        uint256 size = users.length;
        uint256 length = assets.length;
        usersDataWithReserves = new UserDataWithReserves[](size);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            (uint256 totalCollateralETH, uint256 totalBorrowsETH, uint256 healthFactor) = _aaveV1GetUserData(user);

            UserReserve[] memory userReserves = new UserReserve[](length);

            for (uint256 j; j < length; j++) {
                address asset = assets[j];

                (uint256 currentATokenBalance, uint256 currentBorrowBalance, , ) = core.getUserBasicReserveData(
                    asset,
                    user
                );

                userReserves[j] = UserReserve({
                    asset: asset,
                    collateralBalance: currentATokenBalance,
                    borrowBalance: currentBorrowBalance
                });
            }

            usersDataWithReserves[i] = UserDataWithReserves({
                userData: UserData({
                    addr: user,
                    totalCollateralETH: totalCollateralETH,
                    totalBorrowsETH: totalBorrowsETH,
                    healthFactor: healthFactor
                }),
                reserves: userReserves
            });
        }

        return usersDataWithReserves;
    }

    function aaveV2GetGlobalReservesData() external view returns (AAVEReservesData[] memory reservesData) {
        ILendingPoolV2 pool = ILendingPoolV2(V2_POOL);
        IProtocolDataProviderV2 dataProvider = IProtocolDataProviderV2(V2_DATA_PROVIDER);

        IOracle oracle = IOracle(V2_ORACLE);

        address[] memory assets = pool.getReservesList();

        uint256 length = assets.length;

        reservesData = new AAVEReservesData[](assets.length);

        for (uint256 i; i < length; i++) {
            address asset = assets[i];
            uint price = oracle.getAssetPrice(asset);

            (uint256 decimals, , , uint256 liquidationBonus, , , , , , ) = dataProvider.getReserveConfigurationData(
                asset
            );

            reservesData[i] = AAVEReservesData({
                asset: asset,
                price: price,
                decimals: decimals,
                bonus: liquidationBonus
            });
        }

        return reservesData;
    }

    function aaveV2GetUsersData(address[] calldata users) external view returns (UserData[] memory usersData) {
        uint256 size = users.length;
        usersData = new UserData[](size);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            (uint256 totalCollateralETH, uint256 totalDebtETH, uint256 healthFactor) = _aaveV2GetUserData(user);

            usersData[i] = UserData({
                addr: user,
                totalCollateralETH: totalCollateralETH,
                totalBorrowsETH: totalDebtETH,
                healthFactor: healthFactor
            });
        }

        return usersData;
    }

    function aaveV2GetUsersDataWithReserves(
        address[] calldata users
    ) external view returns (UserDataWithReserves[] memory usersDataWithReserves) {
        ILendingPoolV2 pool = ILendingPoolV2(V2_POOL);
        IProtocolDataProviderV2 dataProvider = IProtocolDataProviderV2(V2_DATA_PROVIDER);

        address[] memory assets = pool.getReservesList();

        uint256 size = users.length;
        uint256 length = assets.length;
        usersDataWithReserves = new UserDataWithReserves[](size);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            (uint256 totalCollateralETH, uint256 totalDebtETH, uint256 healthFactor) = _aaveV2GetUserData(user);

            DataTypes.UserConfigurationMap memory config = pool.getUserConfiguration(user);
            UserReserve[] memory userReserves = new UserReserve[](length);

            for (uint256 j; j < length; j++) {
                address asset = assets[j];

                bool isUsingAsCollateralOrBorrowing = (config.data >> (j * 2)) & 3 != 0;
                bool usageAsCollateralEnabled;
                uint256 currentATokenBalance;
                uint256 currentStableDebt;
                uint256 currentVariableDebt;

                if (isUsingAsCollateralOrBorrowing) {
                    (
                        currentATokenBalance,
                        currentStableDebt,
                        currentVariableDebt,
                        ,
                        ,
                        ,
                        ,
                        ,
                        usageAsCollateralEnabled
                    ) = dataProvider.getUserReserveData(asset, user);
                }

                userReserves[j] = UserReserve({
                    asset: asset,
                    borrowBalance: currentStableDebt + currentVariableDebt,
                    collateralBalance: usageAsCollateralEnabled ? currentATokenBalance : 0
                });
            }

            usersDataWithReserves[i] = UserDataWithReserves({
                userData: UserData({
                    addr: user,
                    totalCollateralETH: totalCollateralETH,
                    totalBorrowsETH: totalDebtETH,
                    healthFactor: healthFactor
                }),
                reserves: userReserves
            });
        }

        return usersDataWithReserves;
    }

    function aaveV3GetGlobalReservesData() external view returns (AAVEV3ReservesData[] memory reservesData) {
        ILendingPoolV3 pool = ILendingPoolV3(V3_POOL);
        IProtocolDataProviderV3 dataProvider = IProtocolDataProviderV3(V3_DATA_PROVIDER);

        IOracle oracle = IOracle(V3_ORACLE);

        address[] memory assets = pool.getReservesList();

        uint256 length = assets.length;

        reservesData = new AAVEV3ReservesData[](length);

        uint256 ethPrice = oracle.getAssetPrice(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);

        for (uint256 i; i < length; i++) {
            address asset = assets[i];
            uint256 price = oracle.getAssetPrice(asset);

            uint256 config = pool.getConfiguration(asset).data;

            (uint256 decimals, , , uint256 liquidationBonus, , , , , , ) = dataProvider.getReserveConfigurationData(
                asset
            );

            reservesData[i] = AAVEV3ReservesData({
                asset: asset,
                ethPrice: (price * 10 ** 18) / ethPrice,
                nativePrice: price,
                decimals: decimals,
                bonus: liquidationBonus,
                fee: getLiquidationProtocolFee(config)
            });
        }

        return reservesData;
    }

    function aaveV3GetUsersData(address[] calldata _users) external view returns (UserData[] memory usersData) {
        uint256 size = _users.length;
        usersData = new UserData[](size);

        uint256 ethPrice = IOracle(V3_ORACLE).getAssetPrice(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);

        for (uint256 i; i < size; i++) {
            address user = _users[i];

            (uint256 totalCollateralBase, uint256 totalDebtBase, uint256 healthFactor) = _aaveV3GetUserData(
                user,
                ethPrice
            );

            usersData[i] = UserData({
                addr: user,
                totalCollateralETH: totalCollateralBase,
                totalBorrowsETH: totalDebtBase,
                healthFactor: healthFactor
            });
        }

        return usersData;
    }

    function aaveV3GetUsersDataWithReserves(
        address[] calldata users
    ) external view returns (UserDataWithReserves[] memory usersDataWithReserves) {
        ILendingPoolV3 pool = ILendingPoolV3(V3_POOL);
        IProtocolDataProviderV3 dataProvider = IProtocolDataProviderV3(V3_DATA_PROVIDER);

        address[] memory assets = pool.getReservesList();

        uint256 size = users.length;
        uint256 length = assets.length;
        usersDataWithReserves = new UserDataWithReserves[](size);

        uint256 ethPrice = IOracle(V3_ORACLE).getAssetPrice(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            (uint256 totalCollateralBase, uint256 totalDebtBase, uint256 healthFactor) = _aaveV3GetUserData(
                user,
                ethPrice
            );

            DataTypes.UserConfigurationMap memory config = pool.getUserConfiguration(user);
            UserReserve[] memory userReserves = new UserReserve[](length);

            for (uint256 j; j < length; j++) {
                address asset = assets[j];

                bool isUsingAsCollateralOrBorrowing = (config.data >> (j << 1)) & 3 != 0;
                bool usageAsCollateralEnabled;
                uint256 currentATokenBalance;
                uint256 currentStableDebt;
                uint256 currentVariableDebt;

                if (isUsingAsCollateralOrBorrowing) {
                    (
                        currentATokenBalance,
                        currentStableDebt,
                        currentVariableDebt,
                        ,
                        ,
                        ,
                        ,
                        ,
                        usageAsCollateralEnabled
                    ) = dataProvider.getUserReserveData(asset, user);
                }

                userReserves[j] = UserReserve({
                    asset: asset,
                    borrowBalance: currentStableDebt + currentVariableDebt,
                    collateralBalance: usageAsCollateralEnabled ? currentATokenBalance : 0
                });
            }

            usersDataWithReserves[i] = UserDataWithReserves({
                userData: UserData({
                    addr: user,
                    totalCollateralETH: totalCollateralBase,
                    totalBorrowsETH: totalDebtBase,
                    healthFactor: healthFactor
                }),
                reserves: userReserves
            });
        }

        return usersDataWithReserves;
    }

    function compoundGetGlobalReservesData() external view returns (CompoundReservesData[] memory reservesData) {
        ICompoundController controller = ICompoundController(COMOPUND_CONTROLLER);

        ICompoundOracle oracle = ICompoundOracle(COMOPUND_ORACLE);

        address[] memory markets = controller.getAllMarkets();

        uint256 length = markets.length;

        reservesData = new CompoundReservesData[](length);

        uint256 bonus = controller.liquidationIncentiveMantissa();
        uint256 ethPrice = oracle.price('ETH') * 10 ** 12; // = 10^18

        for (uint256 i; i < length; i++) {
            address market = markets[i];
            uint256 decimals = 18;
            address underlying = 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE;

            if (market != 0x4Ddc2D193948926D02f9B1fE9e1daa0718270ED5) {
                ICompoundToken cToken = ICompoundToken(market);
                underlying = cToken.underlying();
                decimals = IERC20(underlying).decimals();
            }

            uint256 nativePrice = oracle.getUnderlyingPrice(market);
            uint256 price = ((nativePrice / (18 - decimals == 0 ? 1 : 10 ** (18 - decimals))) * 10 ** 18) / ethPrice;

            ICompoundController.Market memory marketData = controller.markets(market);

            reservesData[i] = CompoundReservesData({
                market: market,
                underlying: underlying,
                ethPrice: price,
                nativePrice: nativePrice,
                decimals: decimals,
                bonus: bonus,
                collateralFactor: marketData.collateralFactorMantissa
            });
        }

        return reservesData;
    }

    function compoundGetUsersData(address[] calldata users) external view returns (UserData[] memory userData) {
        uint256 size = users.length;
        userData = new UserData[](size);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            ICompoundController controller = ICompoundController(COMOPUND_CONTROLLER);

            ICompoundToken[] memory assets = controller.getAssetsIn(user);
            uint256 length = assets.length;

            UserReserve[] memory userReserves = new UserReserve[](length);

            for (uint256 j; j < length; j++) {
                ICompoundToken asset = assets[j];

                (, uint256 collateralAmount, uint256 borrowAmount, uint256 exchangeRateMantissa) = asset
                    .getAccountSnapshot(user);

                uint256 collateralBalance = (collateralAmount * exchangeRateMantissa) / 10 ** 18;

                userReserves[j] = UserReserve({
                    asset: address(asset),
                    borrowBalance: borrowAmount,
                    collateralBalance: collateralBalance
                });
            }

            (uint256 healthFactor, uint256 totalBorrowsETH, uint256 totalCollateralETH) = _getCompoundUserData(
                userReserves
            );

            userData[i] = UserData({
                addr: user,
                totalCollateralETH: totalCollateralETH,
                totalBorrowsETH: totalBorrowsETH,
                healthFactor: healthFactor
            });
        }

        return userData;
    }

    function compoundGetUsersDataWithReserves(
        address[] calldata users
    ) external view returns (UserDataWithReserves[] memory usersDataWithReserves) {
        ICompoundController controller = ICompoundController(COMOPUND_CONTROLLER);

        uint256 size = users.length;

        usersDataWithReserves = new UserDataWithReserves[](size);

        for (uint256 i; i < size; i++) {
            address user = users[i];

            ICompoundToken[] memory assets = controller.getAssetsIn(user);
            uint256 length = assets.length;

            UserReserve[] memory userReserves = new UserReserve[](length);

            for (uint256 j; j < length; j++) {
                ICompoundToken asset = assets[j];

                (, uint256 collateralAmount, uint256 borrowAmount, uint256 exchangeRateMantissa) = asset
                    .getAccountSnapshot(user);

                uint256 collateralBalance = (collateralAmount * exchangeRateMantissa) / 10 ** 18;

                userReserves[j] = UserReserve({
                    asset: address(asset),
                    borrowBalance: borrowAmount,
                    collateralBalance: collateralBalance
                });
            }

            (uint256 healthFactor, uint256 totalBorrowsETH, uint256 totalCollateralETH) = _getCompoundUserData(
                userReserves
            );

            usersDataWithReserves[i] = UserDataWithReserves({
                userData: UserData({
                    addr: user,
                    totalCollateralETH: totalCollateralETH,
                    totalBorrowsETH: totalBorrowsETH,
                    healthFactor: healthFactor
                }),
                reserves: userReserves
            });
        }

        return usersDataWithReserves;
    }

    function getLiquidationProtocolFee(uint256 config) public pure returns (uint256) {
        return (config & ~LIQUIDATION_PROTOCOL_FEE_MASK) >> 152;
    }
}
