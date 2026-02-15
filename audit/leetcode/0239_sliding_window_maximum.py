# LeetCode 239: Sliding Window Maximum
# Python version

def maxSlidingWindow(nums: List[int], k: int) -> List[int]:
    output = []
    q = collections.deque()  # index
    l = r = 0
    # O(n) O(n)
    while r < len(nums):
        # pop smaller values from q
        while q and nums[q[-1]] < nums[r]:
            q.pop()
        q.append(r)

        # remove left val from window
        if l > q[0]:
            q.popleft()

        if (r + 1) >= k:
            output.append(nums[q[0]])
            l += 1
        r += 1

    return output



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
