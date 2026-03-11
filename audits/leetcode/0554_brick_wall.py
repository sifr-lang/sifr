# LeetCode 554: Brick Wall
# Python version

def leastBricks(wall: List[List[int]]) -> int:
    countGap = { 0 : 0 }    # { Position : Gap count }

    for r in wall:
        total = 0   # Position
        for b in r[:-1]:
            total += b
            countGap[total] = 1 + countGap.get(total, 0)

    return len(wall) - max(countGap.values())    # Total number of rows - Max gap



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
