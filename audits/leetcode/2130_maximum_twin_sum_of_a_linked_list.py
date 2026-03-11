# LeetCode 2130: Maximum Twin Sum Of A Linked List
# Python version

def pairSum(head: Optional[ListNode]) -> int:
    slow, fast = head, head
    prev = None
    while fast and fast.next:
        fast = fast.next.next
        tmp = slow.next
        slow.next = prev
        prev = slow
        slow = tmp

    res = 0
    while slow:
        res = max(res, prev.val + slow.val)
        prev = prev.next
        slow = slow.next
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
