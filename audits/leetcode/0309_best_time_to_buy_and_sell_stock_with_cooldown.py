# LeetCode 309: Best Time To Buy And Sell Stock With Cooldown
# Python version

def maxProfit(prices: List[int]) -> int:
    # State: Buying or Selling?
    # If Buy -> i + 1
    # If Sell -> i + 2

    dp = {}  # key=(i, buying) val=max_profit

    def dfs(i, buying):
        if i >= len(prices):
            return 0
        if (i, buying) in dp:
            return dp[(i, buying)]

        cooldown = dfs(i + 1, buying)
        if buying:
            buy = dfs(i + 1, not buying) - prices[i]
            dp[(i, buying)] = max(buy, cooldown)
        else:
            sell = dfs(i + 2, not buying) + prices[i]
            dp[(i, buying)] = max(sell, cooldown)
        return dp[(i, buying)]

    return dfs(0, True)



def main():
    assert maxProfit([1,2,3,0,2]) == 3
    assert maxProfit([1]) == 0

if __name__ == "__main__":
    main()
