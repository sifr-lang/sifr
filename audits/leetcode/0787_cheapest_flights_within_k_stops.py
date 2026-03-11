# LeetCode 787: Cheapest Flights Within K Stops
# Python version

def findCheapestPrice(
    self, n: int, flights: List[List[int]], src: int, dst: int, k: int
) -> int:
    prices = [float("inf")] * n
    prices[src] = 0

    for i in range(k + 1):
        tmpPrices = prices.copy()

        for s, d, p in flights:  # s=source, d=dest, p=price
            if prices[s] == float("inf"):
                continue
            if prices[s] + p < tmpPrices[d]:
                tmpPrices[d] = prices[s] + p
        prices = tmpPrices
    return -1 if prices[dst] == float("inf") else prices[dst]



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
