
# LeetCode 787: Cheapest Flights Within K Stops
# Python version

def findCheapestPrice(
    self, n: int, flights: list[list[int]], src: int, dst: int, k: int
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
    assert (
        findCheapestPrice(
            None,
            4,
            [[0, 1, 100], [1, 2, 100], [2, 0, 100], [1, 3, 600], [2, 3, 200]],
            0,
            3,
            1,
        )
        == 700
    )
    assert (
        findCheapestPrice(
            None,
            3,
            [[0, 1, 100], [1, 2, 100], [0, 2, 500]],
            0,
            2,
            1,
        )
        == 200
    )
    assert (
        findCheapestPrice(
            None,
            3,
            [[0, 1, 100], [1, 2, 100], [0, 2, 500]],
            0,
            2,
            0,
        )
        == 500
    )

if __name__ == "__main__":
    main()
