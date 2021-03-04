#include<bits/stdc++.h>
using namespace std;
#define pb push_back
#define st first
#define nd second
#define db(x) cerr << #x << " = " << x << endl
#define _ << ", " << 
typedef long long ll;

int main() {
  cin.tie(0), ios_base::sync_with_stdio(false);
  ll w1, h1, w2, h2;
  cin >> w1 >> h1 >> w2 >> h2;
  cout << 2 * (h1 + h2 + 2) + 3 * w1 << endl;
}
