array = [-7, -3, -2, 5, 8]
"-7 -3 -2 5 8"


def find(ar):
    if not any(ar):
        return
    if sum(ar) == 0:
        print('Found ', ar)
        return

    for i, x in enumerate(ar):
        if x == 0:
            continue
        ar[i] = 0
        find(ar)
        ar[i] = x


find(array)
