import heapq
from collections import defaultdict

# LeetCode 355: Design Twitter
# Python version

class Twitter:
    def __init__(self):
        self.count = 0
        self.tweetMap = defaultdict(list)  # userId -> list of [count, tweetIds]
        self.followMap = defaultdict(set)  # userId -> set of followeeId
    def postTweet(self, userId: int, tweetId: int) -> None:
        self.tweetMap[userId].append([self.count, tweetId])
        self.count -= 1
    def getNewsFeed(self, userId: int) -> list[int]:
        res = []
        minHeap = []
        self.followMap[userId].add(userId)
        for followeeId in self.followMap[userId]:
            if followeeId in self.tweetMap:
                index = len(self.tweetMap[followeeId]) - 1
                count, tweetId = self.tweetMap[followeeId][index]
                heapq.heappush(minHeap, [count, tweetId, followeeId, index - 1])
        while minHeap and len(res) < 10:
            count, tweetId, followeeId, index = heapq.heappop(minHeap)
            res.append(tweetId)
            if index >= 0:
                count, tweetId = self.tweetMap[followeeId][index]
                heapq.heappush(minHeap, [count, tweetId, followeeId, index - 1])
        return res
    def follow(self, followerId: int, followeeId: int) -> None:
        self.followMap[followerId].add(followeeId)
    def unfollow(self, followerId: int, followeeId: int) -> None:
        if followeeId in self.followMap[followerId]:
            self.followMap[followerId].remove(followeeId)

def main():
    obj = Twitter()
    obj.postTweet(1, 5)
    assert obj.getNewsFeed(1) == [5]
    obj.follow(1, 2)
    obj.postTweet(2, 6)
    assert obj.getNewsFeed(1) == [6, 5]
    obj.unfollow(1, 2)
    assert obj.getNewsFeed(1) == [5]

if __name__ == "__main__":
    main()
