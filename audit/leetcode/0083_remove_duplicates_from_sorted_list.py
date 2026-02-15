# LeetCode 83: Remove Duplicates From Sorted List
# Python version

def deleteDuplicates(head: Optional[ListNode]) -> Optional[ListNode]:
    cur = head
    while cur:
        while cur.next and cur.next.val == cur.val:
            cur.next = cur.next.next
        cur = cur.next
    return head



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
