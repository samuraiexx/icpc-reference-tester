#include<bits/stdc++.h>

using namespace std;
typedef long long ll;

const int N = 2e6;

ll bit[N];

void add(int p, int v) {
  for (p += 2; p < N; p += p & -p) bit[p] += v;
}

ll query(int p) {
  ll r = 0;
  for (p += 2; p; p -= p & -p) r += bit[p];
  return r;
}

int main() {
  int n;
  cin >> n;

  for (int i = 1; i <= n; i++) {
    int x;
    cin >> x;
    add(i, x);
  }

  int q;
  cin >> q;

  for (int i = 0; i < q; i++) {
    char c;
    int a, b;

    cin >> c >> a >> b;

    if (c == 'q') {
      cout << query(b) - query(a - 1) << '\n';
    }
    else {
      add(a, b);
    }
  }
}