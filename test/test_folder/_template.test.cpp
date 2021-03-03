// @problem_url: https://codeforces.com/contest/1083/problem/E

#include <bits/stdc++.h>
using namespace std;

// @include: test/540A.cpp

int main() {
  int n;
  vector<int> v;

  scanf("%d", &n);
  v.resize(n);
  for (int i = 0; i < n; i++) scanf("%d", &v[i]);

  Segtree seg;
  seg.build(v);

  int q;
  scanf("%d", &q);
  for (int i = 0; i < q; i++) {
    int l, r;
    scanf("%d %d", &l, &r);
    printf("%d\n", seg.query(l, r));
  }

  return 0;
}
