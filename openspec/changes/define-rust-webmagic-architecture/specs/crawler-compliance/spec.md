## ADDED Requirements

### Requirement: Engine must fetch and cache robots rules before crawling a new domain
引擎 SHALL 在首次接触新域名时优先获取并解析该域名的 `robots.txt`，并将结果缓存在对应 Domain Dispatcher 中。

#### Scenario: 首次抓取未知域名
- **WHEN** Spider 第一次遇到某个新域名
- **THEN** 系统在正式抓取页面前先获取并解析该域名的 `robots.txt`

### Requirement: Engine must honor crawl-delay by updating domain rate limits
引擎 SHALL 根据 `robots.txt` 中的 `Crawl-delay` 指令动态调整该域名的限流速率。

#### Scenario: robots 指定 crawl-delay
- **WHEN** 某个域名的 robots 规则包含 `Crawl-delay`
- **THEN** 对应 Domain Dispatcher 的限流器按该延迟更新抓取速率

### Requirement: Framework must provide native sitemap discovery support
框架 SHALL 提供原生 Sitemap 发现与解析能力，并支持递归处理 `sitemapindex`。

#### Scenario: 通过 sitemap 引导抓取高价值页面
- **WHEN** 用户启用 Sitemap 处理能力
- **THEN** 系统能够解析 `sitemapindex` 与子 sitemap，并将发现的 URL 送入调度链路

