# LeetCode 122: Best Time To Buy And Sell Stock Ii
# Python version

def maxProfit(prices: List[int]) -> int:
    max_profit = 0
    for i in range(1, len(prices)):
        if prices[i] > prices[i-1]:
            max_profit += prices[i] - prices[i-1]
    return max_profit



def main():
    assert maxProfit([7,1,5,3,6,4]) == 7
    assert maxProfit([1,2,3,4,5]) == 4
    assert maxProfit([7,6,4,3,1]) == 0

if __name__ == "__main__":
    main()
