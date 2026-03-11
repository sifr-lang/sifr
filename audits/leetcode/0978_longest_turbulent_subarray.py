# LeetCode 978: Longest Turbulent Subarray
# Python version

def maxTurbulenceSize(arr: List[int]) -> int:
    l, r = 0, 1
    res, prev = 1, ""

    while r < len(arr):
        if arr[r - 1] > arr[r] and prev != ">":
            res = max(res, r - l + 1)
            r += 1
            prev = ">"
        elif arr[r - 1] < arr[r] and prev != "<":
            res = max(res, r - l + 1)
            r += 1
            prev = "<"
        else:
            r = r + 1 if arr[r] == arr[r - 1] else r
            l = r - 1
            prev = ""
    return res



def main():
    print(maxTurbulenceSize([9,4,2,10,7,8,8,1,9]))
    print(maxTurbulenceSize([100]))

if __name__ == "__main__":
    main()
