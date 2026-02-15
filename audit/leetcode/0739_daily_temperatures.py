# LeetCode 739: Daily Temperatures
# Python version

def dailyTemperatures(temperatures: List[int]) -> List[int]:
    res = [0] * len(temperatures)
    stack = []  # pair: [temp, index]

    for i, t in enumerate(temperatures):
        while stack and t > stack[-1][0]:
            stackT, stackInd = stack.pop()
            res[stackInd] = i - stackInd
        stack.append((t, i))
    return res



def main():
    print(dailyTemperatures([73,74,75,71,69,72,76,73]))

if __name__ == "__main__":
    main()
