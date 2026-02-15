# LeetCode 2483: Minimum Penalty For A Shop
# Python version

def bestClosingTime(customers: str) -> int:
    
    curPenalty = res = minPenalty = 0

    for i, ele in enumerate(customers):
        if ele == 'Y':
            curPenalty -= 1
            if curPenalty < minPenalty:
                res = i+1
                curPenalty = minPenalty
        else:
            curPenalty += 1

    return res



def main():
    print(bestClosingTime("YYNY"))
    print(bestClosingTime("NNNNN"))

if __name__ == "__main__":
    main()
