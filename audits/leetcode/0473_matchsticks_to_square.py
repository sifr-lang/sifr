# LeetCode 473: Matchsticks To Square
# Python version

def makesquare(matchsticks: List[int]) -> bool:
    length = sum(matchsticks) // 4
    sides = [0] * 4

    if sum(matchsticks) / 4 != length:
        return False
    matchsticks.sort(reverse=True)

    def backtrack(i):
        if i == len(matchsticks):
            return True

        for j in range(4):
            if sides[j] + matchsticks[i] <= length:
                sides[j] += matchsticks[i]
                if backtrack(i + 1):
                    return True
                sides[j] -= matchsticks[i]
        return False

    return backtrack(0)



def main():
    assert makesquare([1, 1, 2, 2, 2]) == True
    assert makesquare([3, 3, 3, 3, 4]) == False

if __name__ == "__main__":
    main()
