# LeetCode 347: Top K Frequent Elements
# Python version

def topKFrequent(nums: List[int], k: int) -> List[int]:
    count = {}
    freq = [[] for i in range(len(nums) + 1)]

    for n in nums:
        count[n] = 1 + count.get(n, 0)
    for n, c in count.items():
        freq[c].append(n)

    res = []
    for i in range(len(freq) - 1, 0, -1):
        res += freq[i]
        if len(res) == k:
            return res
            

    # O(n)



def main():
    assert topKFrequent([1, 1, 1, 2, 2, 3], 2) == [1, 2]
    assert topKFrequent([1], 1) == [1]
    assert topKFrequent([1, 2, 1, 2, 1, 2, 3, 1, 3, 2], 2) == [1, 2]

if __name__ == "__main__":
    main()
