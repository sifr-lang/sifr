
# LeetCode 92: Reverse Linked List Ii
# Python version
from helpers.list_node import ListNode, list_node_to_string
def reverseBetween(
    self, head: ListNode | None, left: int, right: int
) -> ListNode | None:
    dummy = ListNode(0, head)

    # 1) reach node at position "left"
    leftPrev, cur = dummy, head
    for i in range(left - 1):
        leftPrev, cur = cur, cur.next

    # Now cur="left", leftPrev="node before left"
    # 2) reverse from left to right
    prev = None
    for i in range(right - left + 1):
        tmpNext = cur.next
        cur.next = prev
        prev, cur = cur, tmpNext

    # 3) Update pointers
    leftPrev.next.next = cur  # cur is node after "right"
    leftPrev.next = prev  # prev is "right"
    return dummy.next

def main():
    assert list_node_to_string(
        reverseBetween(
            None,
            ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5))))),
            2,
            4,
        )
    ) == "1->4->3->2->5"
    assert list_node_to_string(reverseBetween(None, ListNode(5), 1, 1)) == "5"

if __name__ == "__main__":
    main()
