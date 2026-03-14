
# LeetCode 121: Best Time to Buy and Sell Stock
# Python version with test cases

def maxProfit(prices: list[int]) -> int:
    min_price: int = prices[0]
    max_profit: int = 0
    for i in range(1, len(prices)):
        if prices[i] < min_price:
            min_price = prices[i]
        elif prices[i] - min_price > max_profit:
            max_profit = prices[i] - min_price
    return max_profit

def main():
    assert maxProfit([7, 1, 5, 3, 6, 4]) == 5
    assert maxProfit([7, 6, 4, 3, 1]) == 0
    assert maxProfit([2, 4, 1]) == 2
    assert maxProfit([1, 2]) == 1

if __name__ == "__main__":
    main()
