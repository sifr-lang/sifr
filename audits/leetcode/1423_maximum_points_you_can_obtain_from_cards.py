# LeetCode 1423: Maximum Points You Can Obtain From Cards
# Python version

def maxScore(cardPoints: List[int], k: int) -> int:
    n = len(cardPoints)

    score = maxScore = sum(cardPoints[:k])

    for i in range(1, k + 1):
        score += cardPoints[-i] - cardPoints[k - i]
        maxScore = max(maxScore, score)

    return maxScore



def main():
    print(maxScore([1,2,3,4,5,6,1], 3))

if __name__ == "__main__":
    main()
