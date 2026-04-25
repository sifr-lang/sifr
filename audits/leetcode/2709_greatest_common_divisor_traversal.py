
# LeetCode 2709: Greatest Common Divisor Traversal
# Python version

from helpers.dsu import UnionFind

def canTraverseAllPairs(nums: list[int]) -> bool:
    uf = UnionFind(len(nums))

    factor_index = {}
    for i, n in enumerate(nums):
        f = 2
        while f * f <= n:
            if n % f == 0:
                if f in factor_index:
                    uf.union(i, factor_index[f])
                else:
                    factor_index[f] = i
                while n % f == 0:
                    n = n // f
            f += 1
        if n > 1:
            if n in factor_index:
                uf.union(i, factor_index[n])
            else:
                factor_index[n] = i
    return uf.count == 1
def main():
    assert canTraverseAllPairs([2, 3, 6]) == True
    assert canTraverseAllPairs([3, 9, 5]) == False
    assert canTraverseAllPairs([4, 3, 12, 8]) == True

if __name__ == "__main__":
    main()
