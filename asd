#include<iostream>
#include<vector>
#include<algorithm>
#include<iomanip>
#include<queue>
#include<set>
#include<map>
using namespace std;
#define OVERLOAD_REP(_1, _2, _3, name, ...) name
#define REP1(i, n) for (auto i = std::decay_t<decltype(n)>{}; (i) != (n); ++(i))
#define REP2(i, l, r) for (auto i = (l); (i) != (r); ++(i))
#define rep(...) OVERLOAD_REP(__VA_ARGS__, REP2, REP1)(__VA_ARGS__)
#define REP(i, l, r) rep(i, l, r+1)
#define all(x) (x).begin(),(x).end()
#define rall(x) (x).rbegin(),(x).rend()
using ll = long long;
using ld = long double;
using P = pair<ll,ll>;
struct Edge {
    int to; ll w;
};
using Graph = vector<vector<int> >;
//using Graph = vector<vector<Edge> >;
const ll INF = 2e18;
//const int INF = 2e9;
template<class T> using vc = vector<T>;
template<class T> using vv = vector<vector<T> >;
template<class T> using pq = priority_queue<T>;
template<class T> using pq_g = priority_queue<T, vc<T>, greater<T> >;
template<class T> istream& operator>>(istream& i, vc<T>& v) { rep(j, 0, v.size()) i>>v[j]; return i; }
template<class T> ostream& operator<<(ostream& o, vc<T>& v) { rep(j, 0, v.size()) o<<v[j]<<" "; return o; }
template<class T> bool chmin(T& a, T b) {
    if(a > b) {
        a = b;
        return true;
    }
    return false;
}
template<class T> bool chmax(T& a, T b) {
    if(a < b) {
        a = b;
        return true;
    }
    return false;
}

int main() {
    // 高速化
    ios::sync_with_stdio(false);
    cin.tie(nullptr);

    // 小数点の出力桁数を指定
    cout << fixed << setprecision(10);

    // メイン
    int N;
    cin >> N;
    vc<int> a(N);
    rep(i, 0, N) cin >> a[i];
    


    return 0;
}
