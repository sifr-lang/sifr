
# LeetCode 2709: Greatest Common Divisor Traversal
# Python version

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


class UnionFind:
    def __init__(self, n):
        self.par = [i for i in range(n)]
        self.size = [1] * n
        self.count = n
    def find(self, x):
        if self.par[x] != x:
            self.par[x] = self.find(self.par[x])
        return self.par[x]
    def union(self, x, y):
        px, py = self.find(x), self.find(y)
        if px == py:
            return
        if self.size[px] < self.size[py]:
            self.par[px] = py
            self.size[py] += self.size[px]
        else:
            self.par[py] = px
            self.size[px] += self.size[py]
        self.count -=1

def main():
    assert canTraverseAllPairs([2, 3, 6]) == True
    assert canTraverseAllPairs([3, 9, 5]) == False
    assert canTraverseAllPairs([4, 3, 12, 8]) == True

if __name__ == "__main__":
    main()
