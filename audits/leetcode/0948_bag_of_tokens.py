
# LeetCode 948: Bag Of Tokens
# Python version

def bagOfTokensScore(tokens: list[int], power: int) -> int:
    res = score = 0
    tokens.sort()

    l, r = 0, len(tokens) - 1
    while (l <= r):
        if power >= tokens[l]:
            power -= tokens[l]
            l += 1
            score += 1
            res = max(res, score)
        elif score > 0:
            power += tokens[r]
            r -= 1
            score -= 1
        else:
            break
    return res



def main():
    assert bagOfTokensScore([100], 50) == 0
    assert bagOfTokensScore([200,100], 150) == 1

if __name__ == "__main__":
    main()
