// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./interfaces/IUniswapV2Pair.sol";
import "./interfaces/IUniswapV3Pool.sol";
import "./interfaces/IERC20.sol";

contract Router {
    /**
     * @notice Performs a token swap on a v2 pool
     * @return afterBalance Post-swap balance, accounting for token tax
     */
    function calculateSwapV2(uint256 amountIn, address pair, address inputToken, address outputToken)
        external
        returns (uint256 afterBalance)
    {
        IERC20(inputToken).transfer(pair, amountIn);

        uint256 beforeBalance = IERC20(outputToken).balanceOf(address(this));

        uint256 reserveIn;
        uint256 reserveOut;

        {
            // Avoid stack too deep error
            (uint256 reserve0, uint256 reserve1,) = IUniswapV2Pair(pair).getReserves();

            // sort reserves
            if (inputToken < outputToken) {
                reserveIn = reserve0;
                reserveOut = reserve1;
            } else {
                reserveIn = reserve1;
                reserveOut = reserve0;
            }
        }

        uint256 actualAmountIn = IERC20(inputToken).balanceOf(address(pair)) - reserveIn;
        uint256 amountOut = _getAmountOut(actualAmountIn, reserveIn, reserveOut);

        (uint256 amount0Out, uint256 amount1Out) =
            inputToken < outputToken ? (uint256(0), amountOut) : (amountOut, uint256(0));
        IUniswapV2Pair(pair).swap(amount0Out, amount1Out, address(this), new bytes(0));

        afterBalance = beforeBalance - IERC20(outputToken).balanceOf(address(this));
    }

    /**
     * @notice Performs a token swap on a v3 pool
     * @return amountOut the amount of the token that would be received
     */
    function calculateSwapV3(int256 amountIn, address pool, address inputToken, address outputToken)
        public
        returns (uint256 amountOut)
    {
        IUniswapV3Pool targetPool = IUniswapV3Pool(pool);

        bool zeroForOne = inputToken < outputToken;
        uint160 sqrtPriceLimitX96 = (zeroForOne ? 4295128749 : 1461446703485210103287273052203988822378723970341);

        // Data for callback
        bytes memory data = abi.encode(zeroForOne, inputToken);

        (int256 amount0, int256 amount1) = targetPool.swap(address(this), zeroForOne, amountIn, sqrtPriceLimitX96, data);

        amountOut = uint256(-(zeroForOne ? amount1 : amount0));
    }

    /**
     * @notice Performs a token swap on a v3 pool
     * @return amountIn the amount of the token that would be send
     */
     function calculateSwapV3AmountIn(int256 amountOut, address pool, address inputToken, address outputToken)
        public
        returns (uint256 amountIn)
    {
        IUniswapV3Pool targetPool = IUniswapV3Pool(pool);

        bool zeroForOne = inputToken < outputToken;
        uint160 sqrtPriceLimitX96 = (zeroForOne ? 4295128749 : 1461446703485210103287273052203988822378723970341);

        // Data for callback
        bytes memory data = abi.encode(zeroForOne, inputToken);

        (int256 amount0, int256 amount1) = targetPool.swap(address(this), zeroForOne, amountOut, sqrtPriceLimitX96, data);

        amountIn = uint256(zeroForOne ? amount0 : amount1);
    }

    /**
     * @notice Post swap callback to sends amount of input token to v3 pool
     */
    function uniswapV3SwapCallback(int256 amount0Delta, int256 amount1Delta, bytes calldata _data) external {
        require(amount0Delta > 0 || amount1Delta > 0);
        (bool isZeroForOne, address inputToken) = abi.decode(_data, (bool, address));

        if (isZeroForOne) {
            IERC20(inputToken).transfer(msg.sender, uint256(amount0Delta));
        } else {
            IERC20(inputToken).transfer(msg.sender, uint256(amount1Delta));
        }
    }

    /**
     * @notice Helper to find output amount from xy=k
     * @return amountOut Output tokens expected from swap
     */
    function _getAmountOut(uint256 amountIn, uint256 reserveIn, uint256 reserveOut)
        internal
        pure
        returns (uint256 amountOut)
    {
        require(amountIn > 0, "UniswapV2Library: INSUFFICIENT_INPUT_AMOUNT");
        require(reserveIn > 0 && reserveOut > 0, "UniswapV2Library: INSUFFICIENT_LIQUIDITY");
        uint256 amountInWithFee = amountIn * 997;
        uint256 numerator = amountInWithFee * reserveOut;
        uint256 denominator = reserveIn * 1000 + amountInWithFee;
        amountOut = numerator / denominator;
    }
}
