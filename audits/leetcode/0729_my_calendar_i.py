
# LeetCode 729: My Calendar I
# Python version
class MyCalendar:
    def __init__(self):
        self.calendar = CalendarNode(-1, -1)
    def book(self, start: int, end: int) -> bool:
        def bookHelper(cur, targetStart, targetEnd):
            if targetStart > cur.end:
                if not cur.right:
                    cur.right = CalendarNode(targetStart, targetEnd)
                    return True
                return bookHelper(cur.right, targetStart, targetEnd)
            elif targetEnd < cur.start:
                if not cur.left:
                    cur.left = CalendarNode(targetStart, targetEnd)
                    return True
                return bookHelper(cur.left, targetStart, targetEnd)
            return False
        return bookHelper(self.calendar, start, end-1) # "end-1" because "end" bound is exclusive (see example 1) 
class CalendarNode:
    def __init__(self, start, end):
        self.start = start
        self.end = end
        self.left = None
        self.right = None

def main():
    obj = MyCalendar()
    assert obj.book(10, 20) == True
    assert obj.book(15, 25) == False
    assert obj.book(20, 30) == True

if __name__ == "__main__":
    main()
