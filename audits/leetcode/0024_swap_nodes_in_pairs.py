# LeetCode 24: Swap Nodes In Pairs
# Python version

def swapPairs(head: ListNode) -> ListNode:
    dummy = ListNode(0, head)
    prev, curr = dummy, head

    while curr and curr.next:
        # save ptrs
        nxtPair = curr.next.next
        second = curr.next

        # reverse this pair
        second.next = curr
        curr.next = nxtPair
        prev.next = second

        # update ptrs
        prev = curr
        curr = nxtPair

    return dummy.next



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
