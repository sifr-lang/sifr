
# LeetCode 904: 0904_Fruit_Into_Baskets
# Python version

def totalFruit(fruits: list[int]) -> int:
    tr = {}
    l = r = 0
    res = 0
    while r < len(fruits):
        if fruits[r] not in tr:
            tr[fruits[r]] = 1
        else:
            tr[fruits[r]] += 1
        while len(tr) > 2:
            tr[fruits[l]] -= 1
            if tr[fruits[l]] == 0:
                del tr[fruits[l]]
            l += 1
        res = max(res, r-l+1)
        r += 1
    return res



def main():
    assert totalFruit([1, 2, 1]) == 3
    assert totalFruit([0, 1, 2, 2]) == 3
    assert totalFruit([1, 2, 3, 2, 2]) == 4

if __name__ == "__main__":
    main()
