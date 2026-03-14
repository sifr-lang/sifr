
# LeetCode 1603: Design Parking System
# Python version

class ParkingSystem:
    def __init__(self, big: int, medium: int, small: int):
        self.parking = {
            1: [0 ,big],
            2: [0, medium],
            3: [0, small]
        }
    def addCar(self, carType: int) -> bool:
        new_total = self.parking[carType][0] + 1
        if new_total <= self.parking[carType][1]:
            self.parking[carType][0] += 1
            return True
        return False

def main():
    obj = ParkingSystem(1, 1, 0)
    assert obj.addCar(1) == True
    assert obj.addCar(2) == True
    assert obj.addCar(3) == False
    assert obj.addCar(1) == False

if __name__ == "__main__":
    main()
