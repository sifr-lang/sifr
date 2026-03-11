# LeetCode 1985: Find The Kth Largest Integer In The Array
# Python version

def kthLargestNumber(nums: List[str], k: int) -> str:
    maxHeap = [-int(n) for n in nums]
    heapq.heapify(maxHeap)
    while k>1:
        heapq.heappop(maxHeap)
        k-=1
    return str(-maxHeap[0])



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
