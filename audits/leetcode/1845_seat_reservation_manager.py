# LeetCode 1845: Seat Reservation Manager
# Python version

class SeatManager:
    def __init__(self, n: int):
        self.seats = [i for i in range(1, n + 1)]
    def reserve(self) -> int:
        return heapq.heappop(self.seats)
    def unreserve(self, seatNumber: int) -> None:
        heapq.heappush(self.seats, seatNumber)

def main():
    print("no test cases")

if __name__ == "__main__":
    main()
