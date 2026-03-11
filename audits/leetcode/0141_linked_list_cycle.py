# LeetCode 141: Linked List Cycle
# Python version

def hasCycle(head: ListNode) -> bool:
    slow, fast = head, head

    while fast and fast.next:
        slow = slow.next
        fast = fast.next.next
        if slow == fast:
            return True
    return False



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
