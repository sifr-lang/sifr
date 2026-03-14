# LeetCode 2300: Successful Pairs Of Spells And Potions
# Python version

def successfulPairs(spells: List[int], potions: List[int], success: int) -> List[int]:
    pairs = []
    potions.sort()
    n = len(potions)

    for i in range(len(spells)):
        l, r = 0, len(potions) - 1
        
        while l <= r:
            m = (l + r) // 2
            if spells[i] * potions[m] >= success:
                r = m - 1
            else:
                l = m + 1
        if l < len(potions):
            pairs.append(n - l)
        else:
            pairs.append(0)

    return pairs
    
    



def main():
    assert successfulPairs([5,1,3], [1,2,3,4,5], 7) == [4, 0, 3]

if __name__ == "__main__":
    main()
