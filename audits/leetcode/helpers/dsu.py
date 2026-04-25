class UnionFind:
    def __init__(self, n: int | None = None):
        # n=None preserves the sparse dict-backed shape used by fixtures that
        # compute component counts from explicit findParent probes.
        self.f = {}
        self.par = [] if n is None else [i for i in range(n)]
        self.rank = [] if n is None else [1] * n
        self.size = [] if n is None else [1] * n
        self.count = 0 if n is None else n

    def findParent(self, x: int) -> int:
        y = self.f.get(x, x)
        if x != y:
            y = self.f[x] = self.findParent(y)
        return y

    def find(self, x: int) -> int:
        if not self.par:
            return self.findParent(x)
        while x != self.par[x]:
            self.par[x] = self.par[self.par[x]]
            x = self.par[x]
        return x

    def union(self, x: int, y: int) -> bool:
        if not self.par:
            px, py = self.findParent(x), self.findParent(y)
            if px == py:
                return False
            self.f[px] = py
            return True
        px, py = self.find(x), self.find(y)
        if px == py:
            return False
        if self.rank[px] > self.rank[py]:
            self.par[py] = px
            self.rank[px] += self.rank[py]
            self.size[px] += self.size[py]
        else:
            self.par[px] = py
            self.rank[py] += self.rank[px]
            self.size[py] += self.size[px]
        self.count -= 1
        return True
