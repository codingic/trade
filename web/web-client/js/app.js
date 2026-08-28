    const API_BASE = 'http://127.0.0.1:3000';
    const chart = echarts.init(document.getElementById('chart'), 'dark');

    // 各指标面板的 grid 配置（上下堆叠，共 6 个区域）
    // 布局从上到下：K线(含布林带) → 成交量 → RSI → MACD → KDJ → ATR/CCI/OBV
    const grids = [
      { left: 60, right: 20, top: 20, height: '38%' },       // 0: K线 + MA + BOLL
      { left: 60, right: 20, top: '44%', height: '10%' },    // 1: 成交量
      { left: 60, right: 20, top: '56%', height: '10%' },    // 2: RSI
      { left: 60, right: 20, top: '68%', height: '10%' },    // 3: MACD
      { left: 60, right: 20, top: '80%', height: '10%' },    // 4: KDJ
      { left: 60, right: 20, top: '92%', height: '8%' },     // 5: ATR/CCI/OBV
    ];

    // 指标显隐状态表：key = series 的 id，value = 是否显示
    // 这个对象是「唯一的真相源」，按钮点击改这里，再同步到图表
    const visibility = {};
    // 指标定义：{ id, label } —— 用于生成控制按钮，且 id 必须与 series 里的 id 一致
    const indicatorDefs = [
      { id: 'ma7',        label: 'MA7' },
      { id: 'ma25',       label: 'MA25' },
      { id: 'boll',       label: 'BOLL' },
      { id: 'volume',     label: '成交量' },
      { id: 'vol_ma5',    label: 'VOL_MA5' },
      { id: 'vol_ma20',   label: 'VOL_MA20' },
      { id: 'rsi',        label: 'RSI' },
      { id: 'macd',       label: 'MACD' },
      { id: 'kdj',        label: 'KDJ' },
      { id: 'atr',        label: 'ATR' },
      { id: 'cci',        label: 'CCI' },
      { id: 'obv',        label: 'OBV' },
    ];

    // 生成控制按钮
    function buildIndicatorButtons() {
      const panel = document.getElementById('indicatorPanel');
      indicatorDefs.forEach(def => {
        const btn = document.createElement('button');
        btn.className = 'ind-btn on';   // 默认全部显示
        btn.textContent = def.label;
        btn.dataset.id = def.id;
        visibility[def.id] = true;       // 初始化状态：显示
        btn.addEventListener('click', () => toggleIndicator(def.id, btn));
        panel.appendChild(btn);
      });
    }

    // 切换某个指标组的显隐
    function toggleIndicator(id, btn) {
      visibility[id] = !visibility[id];
      btn.classList.toggle('on', visibility[id]);
      applyVisibility();
    }

    // 把 visibility 状态同步到图表的所有 series
    // 注意：ECharts 的 series 没有顶层 show 属性，直接设 show 会被忽略。
    // 正确做法是用 legend.selected（按 series 的 name 匹配）来控制系列显隐。
    function applyVisibility() {
      const selected = {};
      seriesDefs.forEach(def => {
        selected[def.name] = visibility[def.group] !== false;
      });
      chart.setOption({ legend: { selected } });
    }

    // series id → 控制组 的映射（多子项指标的后缀归到主指标组）
    function groupOf(id) {
      if (!id) return null;
      if (id === 'kline') return null;   // K线不可隐藏
      const multiPart = ['boll', 'macd', 'kdj'];
      for (const g of multiPart) {
        if (id.startsWith(g + '_')) return g;
      }
      return id;                          // 单子项指标：id 即组名
    }

    // 所有 series 的 name 及其归属控制组，在 loadData 里维护
    let seriesDefs = [];

    async function fetchJson(url, options = {}) {
      const res = await fetch(url, options);
      const text = await res.text();

      if (!res.ok) {
        const detail = text.trim() ? `: ${text.trim()}` : '';
        throw new Error(`HTTP ${res.status}${detail}`);
      }

      if (!text.trim()) {
        throw new Error(`HTTP ${res.status} 空响应`);
      }

      try {
        return JSON.parse(text);
      } catch {
        throw new Error(`HTTP ${res.status} 返回了非 JSON 响应`);
      }
    }

    function fmtNumber(value, digits = 2) {
      if (value == null || Number.isNaN(Number(value))) return '—';
      return Number(value).toFixed(digits);
    }

    async function loadData() {
      const symbol = document.getElementById('symbol').value;
      const interval = document.getElementById('interval').value;
      const limit = document.getElementById('limit').value;
      setStatus('加载中...');

      try {
        const data = await fetchJson(
          `${API_BASE}/api/klines?symbol=${symbol}&interval=${interval}&limit=${limit}`
        );

        // 时间轴：毫秒时间戳 → 可读时间字符串
        const times = data.times.map(t => {
          const d = new Date(t);
          return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
        });

        // 工具：把 None → null（ECharts 自动跳过空值）
        const opt = v => v.map(x => x ?? null);

        const option = {
          animation: false,
          axisPointer: { link: [{ xAxisIndex: 'all' }] },
          tooltip: { trigger: 'axis', axisPointer: { type: 'cross' } },
          // 图例不显示，但保留 selected 状态用于控制 series 显隐（配合顶部按钮）
          legend: { show: false },
          grid: grids,
          xAxis: grids.map((g, i) => ({
            type: 'category',
            data: times,
            gridIndex: i,
            boundaryGap: true,
            axisLine: { lineStyle: { color: '#30363d' } },
            axisLabel: i === 5 ? { show: true } : { show: false },
          })),
          yAxis: [
            { gridIndex: 0, scale: true, splitLine: { lineStyle: { color: '#1c2128' } } },
            { gridIndex: 1, scale: true, splitLine: { show: false }, splitNumber: 2 },
            { gridIndex: 2, min: 0, max: 100, splitLine: { show: false } },
            { gridIndex: 3, scale: true, splitLine: { show: false } },
            { gridIndex: 4, scale: true, splitLine: { show: false } },
            { gridIndex: 5, scale: true, splitLine: { show: false }, splitNumber: 2 },
          ],
          series: [
            // ========== 主图：K线 + 均线 + 布林带 ==========
            {
              id: 'kline', name: 'K线', type: 'candlestick', xAxisIndex: 0, yAxisIndex: 0,
              data: data.candles,
              itemStyle: {
                color: '#ef232a', color0: '#14b143',
                borderColor: '#ef232a', borderColor0: '#14b143',
              },
            },
            { id: 'ma7', name: 'MA7', type: 'line', xAxisIndex: 0, yAxisIndex: 0, data: opt(data.ma7), showSymbol: false, lineStyle: { width: 1, color: '#f0b90b' } },
            { id: 'ma25', name: 'MA25', type: 'line', xAxisIndex: 0, yAxisIndex: 0, data: opt(data.ma25), showSymbol: false, lineStyle: { width: 1, color: '#1f6feb' } },
            { id: 'boll_upper', name: 'BOLL上轨', type: 'line', xAxisIndex: 0, yAxisIndex: 0, data: opt(data.boll.upper), showSymbol: false, lineStyle: { width: 1, color: '#f78166', type: 'dashed' } },
            { id: 'boll_middle', name: 'BOLL中轨', type: 'line', xAxisIndex: 0, yAxisIndex: 0, data: opt(data.boll.middle), showSymbol: false, lineStyle: { width: 1, color: '#f78166' } },
            { id: 'boll_lower', name: 'BOLL下轨', type: 'line', xAxisIndex: 0, yAxisIndex: 0, data: opt(data.boll.lower), showSymbol: false, lineStyle: { width: 1, color: '#f78166', type: 'dashed' } },

            // ========== 成交量 + 量均线 ==========
            {
              id: 'volume', name: '成交量', type: 'bar', xAxisIndex: 1, yAxisIndex: 1,
              data: data.volume,
              itemStyle: { color: p => p.value >= 0 ? '#ef232a' : '#14b143' },
            },
            { id: 'vol_ma5', name: 'VOL_MA5', type: 'line', xAxisIndex: 1, yAxisIndex: 1, data: opt(data.vol_ma5), showSymbol: false, lineStyle: { width: 1, color: '#f0b90b' } },
            { id: 'vol_ma20', name: 'VOL_MA20', type: 'line', xAxisIndex: 1, yAxisIndex: 1, data: opt(data.vol_ma20), showSymbol: false, lineStyle: { width: 1, color: '#1f6feb' } },

            // ========== RSI ==========
            { id: 'rsi', name: 'RSI14', type: 'line', xAxisIndex: 2, yAxisIndex: 2, data: opt(data.rsi14), showSymbol: false, lineStyle: { width: 1, color: '#9d7bd8' } },

            // ========== MACD ==========
            { id: 'macd_dif', name: 'DIF', type: 'line', xAxisIndex: 3, yAxisIndex: 3, data: opt(data.macd.dif), showSymbol: false, lineStyle: { width: 1, color: '#f0b90b' } },
            { id: 'macd_dea', name: 'DEA', type: 'line', xAxisIndex: 3, yAxisIndex: 3, data: opt(data.macd.dea), showSymbol: false, lineStyle: { width: 1, color: '#1f6feb' } },
            { id: 'macd_hist', name: 'MACD', type: 'bar', xAxisIndex: 3, yAxisIndex: 3, data: opt(data.macd.hist), itemStyle: { color: p => p.value >= 0 ? '#ef232a' : '#14b143' } },

            // ========== KDJ ==========
            { id: 'kdj_k', name: 'K', type: 'line', xAxisIndex: 4, yAxisIndex: 4, data: opt(data.kdj.k), showSymbol: false, lineStyle: { width: 1, color: '#f0b90b' } },
            { id: 'kdj_d', name: 'D', type: 'line', xAxisIndex: 4, yAxisIndex: 4, data: opt(data.kdj.d), showSymbol: false, lineStyle: { width: 1, color: '#1f6feb' } },
            { id: 'kdj_j', name: 'J', type: 'line', xAxisIndex: 4, yAxisIndex: 4, data: opt(data.kdj.j), showSymbol: false, lineStyle: { width: 1, color: '#9d7bd8' } },

            // ========== ATR / CCI / OBV ==========
            { id: 'atr', name: 'ATR14', type: 'line', xAxisIndex: 5, yAxisIndex: 5, data: opt(data.atr14), showSymbol: false, lineStyle: { width: 1, color: '#58a6ff' } },
            { id: 'cci', name: 'CCI20', type: 'line', xAxisIndex: 5, yAxisIndex: 5, data: opt(data.cci20), showSymbol: false, lineStyle: { width: 1, color: '#3fb950' } },
            { id: 'obv', name: 'OBV', type: 'line', xAxisIndex: 5, yAxisIndex: 5, data: opt(data.obv), showSymbol: false, lineStyle: { width: 1, color: '#d29922' } },
          ],
        };

        // 记录所有 series 的 name → 控制组，供显隐切换使用（legend.selected 按 name 匹配）
        seriesDefs = option.series.map(s => ({ name: s.name, group: groupOf(s.id) })).filter(x => x.group);

        chart.setOption(option, true);
        // 重绘后重新应用用户的显隐选择（否则隐藏状态会被重置）
        applyVisibility();
        setStatus(`已加载 ${symbol} ${interval} · ${data.times.length} 根`);
      } catch (e) {
        setStatus('加载失败：' + e.message + '（请确认后端已启动）');
      }
    }

    function setStatus(text) { document.getElementById('status').textContent = text; }

    document.getElementById('refresh').addEventListener('click', loadData);
    document.getElementById('interval').addEventListener('change', loadData);
    document.getElementById('symbol').addEventListener('change', loadData);
    document.getElementById('limit').addEventListener('change', loadData);

    // 窗口缩放自适应
    window.addEventListener('resize', () => chart.resize());

    // 生成指标控制按钮
    buildIndicatorButtons();

    // 首屏加载
    loadData();
    // 每 30 秒自动刷新一次
    setInterval(loadData, 30000);

    // ===== 数据库查看弹窗 =====
    const dbModal = document.getElementById('dbModal');
    const dbBody = document.getElementById('dbBody');
    const catalogModal = document.getElementById('catalogModal');
    const catalogBody = document.getElementById('catalogBody');
    const backtestModal = document.getElementById('backtestModal');
    const backtestBody = document.getElementById('backtestBody');
    const top20ModalEl = document.getElementById('top20Modal');
    const top20Body = document.getElementById('top20Body');
    const backtestCatalog = document.getElementById('backtestCatalog');
    const backtestForm = document.getElementById('backtestForm');
    const backtestRunButton = document.getElementById('btRun');
    const backtestCatalogButton = document.getElementById('btCatalogRun');
    document.getElementById('dbView').addEventListener('click', openDbModal);
    document.getElementById('catalogView').addEventListener('click', openCatalogModal);
    document.getElementById('top10View').addEventListener('click', openTop10Modal);
    document.getElementById('top20').addEventListener('click', openTop20Modal);
    document.getElementById('backtestView').addEventListener('click', openBacktestModal);
    document.getElementById('dbClose').addEventListener('click', closeDbModal);
    document.getElementById('catalogClose').addEventListener('click', closeCatalogModal);
    document.getElementById('top10Close').addEventListener('click', closeTop10Modal);
    document.getElementById('top20Close').addEventListener('click', closeTop20Modal);
    document.getElementById('top20Rescan').addEventListener('click', loadTop20);
    document.getElementById('backtestClose').addEventListener('click', closeBacktestModal);
    document.getElementById('btSync').addEventListener('click', syncBacktestFormWithChart);
    backtestRunButton.addEventListener('click', runBacktest);
    backtestCatalogButton.addEventListener('click', runStrategyCatalog);
    backtestForm.addEventListener('submit', e => {
      e.preventDefault();
      runBacktest();
    });
    // 点击遮罩空白处也可关闭
    dbModal.addEventListener('click', e => { if (e.target === dbModal) closeDbModal(); });
    catalogModal.addEventListener('click', e => { if (e.target === catalogModal) closeCatalogModal(); });
    backtestModal.addEventListener('click', e => { if (e.target === backtestModal) closeBacktestModal(); });
    top20ModalEl.addEventListener('click', e => { if (e.target === top20ModalEl) closeTop20Modal(); });
    // 按 Esc 关闭
    document.addEventListener('keydown', e => {
      if (e.key === 'Escape') {
        closeDbModal();
        closeCatalogModal();
        closeBacktestModal();
        closeTop10Modal();
        closeTop20Modal();
      }
    });

    function closeDbModal() { dbModal.classList.remove('open'); }
    function closeCatalogModal() { catalogModal.classList.remove('open'); }
    function closeBacktestModal() { backtestModal.classList.remove('open'); }

    function syncBacktestFormWithChart() {
      document.getElementById('btSymbol').value = document.getElementById('symbol').value;
      document.getElementById('btInterval').value = document.getElementById('interval').value;
      // 不覆盖limit，默认0=全部数据
    }

    function openCatalogModal() {
      catalogModal.classList.add('open');
      loadStrategies();
    }

    async function loadStrategies() {
      catalogBody.innerHTML = '<div class="bt-empty">加载策略目录中...</div>';
      try {
        const data = await fetchJson(`${API_BASE}/api/strategies`);
        renderStrategyCatalog(data);
      } catch (e) {
        catalogBody.innerHTML = `<div class="bt-empty">加载失败：${e.message}</div>`;
      }
    }

    let catalogStrategies = []; // 缓存策略列表数据

    function renderStrategyCatalog(data) {
      const total = data.total || 0;
      catalogStrategies = data.strategies || [];
      const categories = data.categories || [];

      renderCatalogListView(total, categories, catalogStrategies);
    }

    function renderCatalogListView(total, categories, strategies) {
      let html = `<div style="margin-bottom:12px; display:flex; justify-content:space-between; align-items:center;">
        <span style="color:#8b949e; font-size:13px;">共 <b style="color:#e6edf3;">${total}</b> 个内置策略，点击卡片可编辑参数并回测</span>
      </div>`;

      const grouped = {};
      for (const s of strategies) {
        if (!grouped[s.category]) grouped[s.category] = [];
        grouped[s.category].push(s);
      }

      for (const cat of categories) {
        const items = grouped[cat] || [];
        html += `
          <div style="margin-bottom:16px;">
            <h4 style="font-size:14px; color:#58a6ff; margin-bottom:8px; padding-bottom:6px; border-bottom:1px solid #21262d;">
              ${cat} <span style="color:#8b949e; font-weight:400; font-size:12px;">(${items.length} 个)</span>
            </h4>
            <div style="display:grid; grid-template-columns: repeat(auto-fill, minmax(280px,1fr)); gap:8px;">
        `;
        for (const s of items) {
          html += `
            <div class="strat-card" data-id="${s.id}" style="background:#0d1117; border:1px solid #21262d; border-radius:6px; padding:10px; cursor:pointer; transition:border-color .15s;" onmouseover="this.style.borderColor='#58a6ff'" onmouseout="this.style.borderColor='#21262d'">
              <div style="display:flex; justify-content:space-between; align-items:start; gap:8px;">
                <span style="color:#e6edf3; font-size:13px; font-weight:500;">${s.index}. ${s.name}</span>
                <span style="color:#3fb950; font-size:10px; white-space:nowrap; background:#0d1117; border:1px solid #238636; border-radius:10px; padding:1px 6px;">回测 ›</span>
              </div>
              <div style="color:#8b949e; font-size:11px; margin-top:4px;">${s.description}</div>
              <div style="color:#484f58; font-size:10px; margin-top:4px; font-family:monospace;">LB=${s.lookback} · ${s.id}</div>
            </div>
          `;
        }
        html += `</div></div>`;
      }

      catalogBody.innerHTML = html;

      // 绑定卡片点击事件
      catalogBody.querySelectorAll('.strat-card').forEach(card => {
        card.addEventListener('click', () => {
          const id = card.dataset.id;
          const strat = catalogStrategies.find(s => s.id === id);
          if (strat) openStrategyEditor(strat);
        });
      });
    }

    function openStrategyEditor(strategy, options = {}) {
      const isCompound = !!options.compound;
      const params = strategy.defaultParams || {};
      const schema = strategy.paramSchema || [];

      const inputStyle = 'width:100%; background:#0d1117; color:#e6edf3; border:1px solid #30363d; border-radius:4px; padding:5px 8px; font-size:12px; box-sizing:border-box;';
      const selectStyle = inputStyle;
      let formHtml = '';
      for (const field of schema) {
        const key = field.key;
        const label = field.label;
        const val = key in params ? params[key] : '';
        const fieldType = String(field.type || '').toLowerCase();
        if (fieldType === 'bool') {
          formHtml += `
            <div style="flex:0 0 160px; display:flex; align-items:center; gap:8px;">
              <input type="checkbox" id="sp_${key}" ${val ? 'checked' : ''} style="width:auto; accent-color:#00d4ff;">
              <label for="sp_${key}" style="color:#c9d1d9; font-size:12px; cursor:pointer;">${label}</label>
            </div>`;
        } else {
          formHtml += `
            <div style="flex:0 0 160px;">
              <label for="sp_${key}" style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">${label}</label>
              <input id="sp_${key}" type="number" value="${val}" min="${field.min}" max="${field.max}" step="${field.step}" style="${inputStyle}">
            </div>`;
        }
      }

      // 回测参数默认值：复利模式默认4h/100U/1x/0.0004/120天
      const capVal = isCompound ? 100 : (document.getElementById('btCapital')?.value || 100);
      const qtyVal = isCompound ? 100 : (document.getElementById('btQuantity')?.value || 100);
      const levVal = isCompound ? 1 : (document.getElementById('btLeverage')?.value || 1);
      const feeVal = isCompound ? 0.0004 : (document.getElementById('btFee')?.value || 0.0004);
      const daysVal = options.defaultDays || 120;
      const limVal = document.getElementById('btLimit')?.value || 0;
      const symVal = options.defaultSymbol || (isCompound ? (document.getElementById('symbol')?.value || 'BTCUSDT') : (document.getElementById('btSymbol')?.value || 'BTCUSDT'));
      const intVal = options.defaultInterval || (isCompound ? '4h' : (document.getElementById('btInterval')?.value || '1m'));

      // 回测设置区域HTML
      let settingsHtml;
      if (isCompound) {
        settingsHtml = `
          <div style="display:flex; flex-wrap:wrap; gap:10px; align-items:flex-end;">
            <div style="flex:0 0 140px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">合约</label>
              <input id="sp_symbol" value="${symVal}" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">周期</label>
              <select id="sp_interval" style="${selectStyle}">
                <option value="1h" ${intVal==='1h'?'selected':''}>1h</option>
                <option value="2h" ${intVal==='2h'?'selected':''}>2h</option>
                <option value="4h" ${intVal==='4h'?'selected':''}>4h</option>
                <option value="6h" ${intVal==='6h'?'selected':''}>6h</option>
                <option value="8h" ${intVal==='8h'?'selected':''}>8h</option>
                <option value="12h" ${intVal==='12h'?'selected':''}>12h</option>
                <option value="1d" ${intVal==='1d'?'selected':''}>1d</option>
              </select>
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">初始资金(U)</label>
              <input id="sp_capital" type="number" value="${capVal}" min="1" step="10" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">杠杆倍数</label>
              <input id="sp_leverage" type="number" value="${levVal}" min="0.1" step="0.1" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">手续费率</label>
              <input id="sp_fee" type="number" value="${feeVal}" min="0" step="0.0001" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">回测天数</label>
              <input id="sp_days" type="number" value="${daysVal}" min="1" step="10" style="${inputStyle}">
            </div>
            <div style="flex:0 0 140px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">止盈%(0=不止盈)</label>
              <input id="sp_take_profit" type="number" value="10" min="0" step="1" style="${inputStyle}">
            </div>
          </div>`;
      } else {
        settingsHtml = `
          <div style="display:flex; flex-wrap:wrap; gap:10px; align-items:flex-end;">
            <div style="flex:0 0 140px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">合约</label>
              <input id="sp_symbol" value="${symVal}" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">周期</label>
              <select id="sp_interval" style="${selectStyle}">
                <option value="1m" ${intVal==='1m'?'selected':''}>1m</option>
                <option value="5m" ${intVal==='5m'?'selected':''}>5m</option>
                <option value="15m" ${intVal==='15m'?'selected':''}>15m</option>
                <option value="1h" ${intVal==='1h'?'selected':''}>1h</option>
                <option value="4h" ${intVal==='4h'?'selected':''}>4h</option>
              </select>
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">资金(U)</label>
              <input id="sp_capital" type="number" value="${capVal}" min="1" step="10" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">保证金(U)</label>
              <input id="sp_quantity" type="number" value="${qtyVal}" min="1" step="1" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">杠杆倍数</label>
              <input id="sp_leverage" type="number" value="${levVal}" min="1" step="1" style="${inputStyle}">
            </div>
            <div style="flex:0 0 120px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">手续费率</label>
              <input id="sp_fee" type="number" value="${feeVal}" min="0" step="0.0001" style="${inputStyle}">
            </div>
            <div style="flex:0 0 160px;">
              <label style="color:#8b949e; font-size:11px; display:block; margin-bottom:3px;">K线数量(0=全部)</label>
              <input id="sp_limit" type="number" value="${limVal}" min="0" step="500" style="${inputStyle}">
            </div>
          </div>`;
      }

      const editorTitle = isCompound ? '编辑参数 · 复利模式' : '编辑参数';
      const runBtnText = isCompound ? '▶ 运行复利回测' : '▶ 运行回测';

      catalogBody.innerHTML = `
        <div style="display:flex; flex-direction:column; gap:12px; min-height: calc(100vh - 80px);">
          <!-- 顶部参数面板 -->
          <div style="background:rgba(15,23,42,0.8); border:1px solid rgba(0,212,255,0.25); border-radius:10px; padding:12px 16px; backdrop-filter:blur(10px); box-shadow:0 0 30px rgba(0,212,255,0.08);">
            <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:10px; gap:12px; flex-wrap:wrap;">
              <div style="display:flex; align-items:center; gap:12px;">
                <button id="spBack" style="background:rgba(0,212,255,0.1); color:#00d4ff; border:1px solid rgba(0,212,255,0.3); border-radius:6px; padding:5px 12px; cursor:pointer; font-size:12px; font-weight:600; transition:all 0.2s;">← 返回</button>
                <div>
                  <h3 style="color:#00d4ff; font-size:15px; margin:0; font-weight:700; text-shadow:0 0 10px rgba(0,212,255,0.3);">${strategy.name}</h3>
                  <div style="color:#8b949e; font-size:11px; margin-top:2px;">${strategy.description} · ${strategy.id || strategy.kind} · ${strategy.category}</div>
                </div>
              </div>
              <button id="spRun" style="background:linear-gradient(135deg,#00d4ff 0%,#0066ff 100%); color:#fff; border:none; border-radius:6px; padding:9px 24px; font-size:13px; font-weight:700; cursor:pointer; box-shadow:0 0 20px rgba(0,212,255,0.4); transition:all 0.2s; white-space:nowrap;">${runBtnText}</button>
            </div>
            <div style="display:flex; flex-wrap:wrap; gap:14px; align-items:flex-end;">
              <div style="color:#00d4ff; font-size:11px; font-weight:700; align-self:center; letter-spacing:0.5px;">⚙ 策略参数</div>
              ${formHtml}
              <div style="width:1px; height:28px; background:rgba(0,212,255,0.2); align-self:center;"></div>
              <div style="color:#7c3aed; font-size:11px; font-weight:700; align-self:center; letter-spacing:0.5px;">📊 回测设置</div>
              ${settingsHtml}
            </div>
          </div>
          <!-- 下方结果区（全屏宽度） -->
          <div id="spResult" style="flex:1; overflow-y:auto; min-height:300px;">
            <div class="bt-empty" style="padding:40px 0;">在上方调整参数后点击「运行回测」查看结果。</div>
          </div>
        </div>`;

      // Store compound flag on the button for later use
      document.getElementById('spRun').dataset.compound = isCompound ? '1' : '0';
      document.getElementById('spRun').dataset.kind = strategy.kind || '';
      document.getElementById('spRun').dataset.lookback = strategy.lookback || 0;

      document.getElementById('spBack').addEventListener('click', () => {
        if (isCompound && options.backAction) {
          options.backAction();
        } else {
          loadStrategies();
        }
      });
      document.getElementById('spRun').addEventListener('click', () => {
        runEditorBacktest(strategy, isCompound);
      });
    }

    async function runEditorBacktest(strategy, isCompound) {
      const schema = strategy.paramSchema || [];
      const params = {};
      for (const field of schema) {
        const key = field.key;
        const el = document.getElementById('sp_' + key);
        if (!el) continue;
        const fieldType = String(field.type || '').toLowerCase();
        if (fieldType === 'bool') {
          params[key] = el.checked;
        } else if (fieldType === 'float') {
          params[key] = parseFloat(el.value) || 0;
        } else {
          params[key] = parseInt(el.value) || 0;
        }
      }

      const resultEl = document.getElementById('spResult');
      resultEl.innerHTML = '<div class="bt-empty">' + (isCompound ? '复利回测中...' : '回测中...') + '</div>';

      let endpoint, payload;
      if (isCompound) {
        endpoint = `${API_BASE}/api/backtest/compound`;
        payload = {
          kind: strategy.kind,
          symbol: document.getElementById('sp_symbol').value,
          interval: document.getElementById('sp_interval').value,
          days: parseInt(document.getElementById('sp_days').value) || 120,
          lookback: parseInt(document.getElementById('spRun').dataset.lookback) || strategy.lookback || 0,
          capital: parseFloat(document.getElementById('sp_capital').value) || 100,
          leverage: parseFloat(document.getElementById('sp_leverage').value) || 1,
          fee: parseFloat(document.getElementById('sp_fee').value) || 0.0004,
          take_profit_pct: (() => { const el = document.getElementById('sp_take_profit'); return el && parseFloat(el.value) > 0 ? parseFloat(el.value) : null; })(),
          params: params,
        };
      } else {
        endpoint = `${API_BASE}/api/backtest/custom`;
        payload = {
          strategy_id: strategy.id,
          symbol: document.getElementById('sp_symbol').value,
          interval: document.getElementById('sp_interval').value,
          capital: parseFloat(document.getElementById('sp_capital').value) || 100,
          quantity: parseFloat(document.getElementById('sp_quantity').value) || 100,
          leverage: parseFloat(document.getElementById('sp_leverage').value) || 1,
          fee: parseFloat(document.getElementById('sp_fee').value) || 0.0004,
          limit: parseInt(document.getElementById('sp_limit').value) || 0,
          lookback: strategy.lookback || 50,
          params: params,
        };
      }

      try {
        const data = await fetchJson(endpoint, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });
        renderCustomBacktestResult(data);
      } catch (e) {
        resultEl.innerHTML = `<div class="bt-empty">回测失败：${e.message}</div>`;
      }
    }

    // 保持旧函数名作为别名，兼容其他调用
    async function runCustomBacktest(strategy) {
      return runEditorBacktest(strategy, false);
    }

    function renderCustomBacktestResult(data) {
      const resultEl = document.getElementById('spResult');
      const s = data.strategy || {};
      const params = data.parameters || {};
      const summary = data.summary || {};
      const trades = data.recentTrades || [];
      const usedParams = s.params || {};
      const profitClass = Number(summary.netProfit) >= 0 ? 'positive' : 'negative';

      // 显示当前使用的参数
      let paramsHtml = '';
      for (const [k, v] of Object.entries(usedParams)) {
        paramsHtml += `<span class="bt-pill">${k}=${typeof v === 'boolean' ? (v ? 'true' : 'false') : v}</span>`;
      }

      let html = `
        <div style="margin-bottom:10px;">
          <h3 style="color:#e6edf3; font-size:16px; margin:0 0 6px 0;">${s.name || ''} 回测结果</h3>
          <div style="color:#8b949e; font-size:12px; margin-bottom:8px;">${s.description || ''}</div>
          <div style="display:flex; flex-wrap:wrap; gap:4px; margin-bottom:8px;">${paramsHtml}</div>
        </div>
        <div class="bt-meta">
          <span class="bt-pill">${params.symbol || '—'} · ${params.interval || '—'}</span>
          <span class="bt-pill">样本 ${summary.bars ?? 0} 根</span>
          <span class="bt-pill">Lookback ${params.lookback ?? '—'}</span>
          <span class="bt-pill">时间 ${fmtTime(summary.firstOpenTime)} → ${fmtTime(summary.lastCloseTime)}</span>
        </div>
        <div class="bt-summary">
          <div class="bt-card"><span class="label">净收益</span><span class="value ${profitClass}">${fmtNumber(summary.netProfit)}U</span></div>
          <div class="bt-card"><span class="label">收益率</span><span class="value ${profitClass}">${fmtNumber(summary.returnPct)}%</span></div>
          <div class="bt-card"><span class="label">最终权益</span><span class="value">${fmtNumber(summary.finalEquity)}U</span></div>
          <div class="bt-card"><span class="label">最大回撤</span><span class="value">${fmtNumber(summary.maxDrawdownPct)}%</span></div>
          <div class="bt-card"><span class="label">胜率</span><span class="value">${fmtNumber(summary.winRatePct)}%</span></div>
          <div class="bt-card"><span class="label">交易笔数</span><span class="value">${summary.tradeCount ?? 0}</span></div>
        </div>
        <div class="bt-meta">
          <span class="bt-pill">初始资金 ${fmtNumber(params.capital)}U</span>
          ${params.days ? `<span class="bt-pill">复利全仓</span>` : ''}
          ${!params.days && params.marginPerTrade != null ? `<span class="bt-pill">保证金 ${fmtNumber(params.marginPerTrade, 2)}U</span>` : ''}
          <span class="bt-pill">倍数 ${fmtNumber(params.leverage, 2)}x</span>
          ${!params.days && params.notionalPerTrade != null ? `<span class="bt-pill">名义仓位 ${fmtNumber(params.notionalPerTrade, 2)}U</span>` : ''}
          <span class="bt-pill">手续费 ${fmtNumber(summary.totalFees, 4)}U</span>
          <span class="bt-pill">盈 ${summary.winCount ?? 0} · 亏 ${summary.lossCount ?? 0}</span>
          ${params.days ? `<span class="bt-pill">回测约 ${Math.round(params.days)} 天</span>` : ''}
        </div>`;

      if (!trades.length) {
        html += '<div class="bt-empty">本次回测没有触发任何成交。可尝试调整参数或使用更多数据。</div>';
        resultEl.innerHTML = html;
        return;
      }

      // 交易记录表格放在上面
      html += '<div class="bt-table-wrap"><table class="bt-table"><thead><tr><th>方向</th><th>入场时间</th><th>入场价</th><th>出场时间</th><th>出场价</th><th>净收益</th><th>手续费</th><th>持有K数</th></tr></thead><tbody>';
      for (const trade of trades) {
        const pnlClass = Number(trade.netPnl) >= 0 ? 'positive' : 'negative';
        html += `<tr>
          <td style="color:${trade.side==='LONG'?'#3fb950':'#f85149'};">${trade.side}</td>
          <td>${fmtTime(trade.entryTime)}</td>
          <td>${fmtNumber(trade.entryPrice, 2)}</td>
          <td>${fmtTime(trade.exitTime)}</td>
          <td>${fmtNumber(trade.exitPrice, 2)}</td>
          <td class="${pnlClass}">${fmtNumber(trade.netPnl, 2)}U</td>
          <td>${fmtNumber(trade.fee, 4)}</td>
          <td>${trade.barsHeld}</td>
        </tr>`;
      }
      html += '</tbody></table></div>';

      // K线图放在最下方，全屏高度
      html += '<div id="btChart" style="width:100%; height:calc(100vh - 120px); margin:14px 0 0 0; background:#0d1117; border:1px solid #30363d; border-radius:6px;"></div>';
      resultEl.innerHTML = html;

      // 渲染K线图和买卖点
      renderBacktestChart(params.symbol, params.interval, trades);
    }

    async function renderBacktestChart(symbol, interval, trades) {
      const chartDom = document.getElementById('btChart');
      if (!chartDom) return;

      try {
        // 请求足够的K线数据（1500根上限足够覆盖回测区间）
        const data = await fetchJson(`${API_BASE}/api/klines?symbol=${symbol}&interval=${interval}&limit=1500`);

        const btChart = echarts.init(chartDom, 'dark');

        // 计算K线价格范围，用于限制竖线高度
        let priceMin = Infinity, priceMax = -Infinity;
        for (const k of data.candles) {
          if (k[2] < priceMin) priceMin = k[2];
          if (k[3] > priceMax) priceMax = k[3];
        }
        const yPad = (priceMax - priceMin) * 0.05;
        const yMin = priceMin - yPad;
        const yMax = priceMax + yPad;

        // 构建买卖点竖线标记数据（两点定义，用coord指定坐标，限制高度在K线范围内）
        const tradeLines = [];

        for (const trade of trades) {
          // 入场竖线
          const entryIdx = data.times.findIndex(t => t >= trade.entryTime);
          if (entryIdx >= 0) {
            const lineColor = trade.side === 'LONG' ? '#00ff88' : '#ff5c7a';
            tradeLines.push([
              {
                coord: [entryIdx, yMin],
                lineStyle: { color: lineColor, type: 'dashed', width: 1.5 },
                label: {
                  show: true,
                  position: 'end',
                  formatter: trade.side === 'LONG' ? '买' : '卖',
                  color: lineColor,
                  fontSize: 11,
                  fontWeight: 'bold',
                },
              },
              { coord: [entryIdx, priceMax] },
            ]);
          }
          // 出场竖线
          const exitIdx = data.times.findIndex(t => t >= trade.exitTime);
          if (exitIdx >= 0) {
            tradeLines.push([
              {
                coord: [exitIdx, yMin],
                lineStyle: { color: '#8b949e', type: 'dashed', width: 1.5 },
                label: {
                  show: true,
                  position: 'end',
                  formatter: '平',
                  color: '#8b949e',
                  fontSize: 11,
                  fontWeight: 'bold',
                },
              },
              { coord: [exitIdx, priceMax] },
            ]);
          }
        }

        const option = {
          backgroundColor: '#0d1117',
          animation: false,
          grid: { left: 50, right: 20, top: 30, bottom: 30 },
          xAxis: {
            type: 'category',
            data: data.times.map((t, i) => i),
            axisLine: { lineStyle: { color: '#30363d' } },
            axisLabel: {
              color: '#8b949e',
              fontSize: 10,
              formatter: function(val) {
                const idx = parseInt(val);
                if (idx % Math.floor(data.times.length / 6) === 0) {
                  return fmtTime(data.times[idx]);
                }
                return '';
              }
            },
            splitLine: { show: false },
          },
          yAxis: {
            scale: true,
            min: yMin,
            max: yMax,
            axisLine: { lineStyle: { color: '#30363d' } },
            axisLabel: { color: '#8b949e', fontSize: 10 },
            splitLine: { lineStyle: { color: '#21262d' } },
          },
          tooltip: {
            trigger: 'axis',
            axisPointer: { type: 'cross' },
            backgroundColor: '#161b22',
            borderColor: '#30363d',
            textStyle: { color: '#c9d1d9', fontSize: 11 },
            formatter: function(params) {
              const idx = params[0].dataIndex;
              const time = fmtTime(data.times[idx]);
              const k = data.candles[idx];
              if (!k) return '';
              return `<div style="font-size:11px;">
                <div style="margin-bottom:4px;color:#8b949e;">${time}</div>
                <div>开: <span style="color:#e6edf3;">${k[0].toFixed(2)}</span></div>
                <div>收: <span style="color:${k[1]>=k[0]?'#3fb950':'#f85149'};">${k[1].toFixed(2)}</span></div>
                <div>低: <span style="color:#e6edf3;">${k[2].toFixed(2)}</span></div>
                <div>高: <span style="color:#e6edf3;">${k[3].toFixed(2)}</span></div>
              </div>`;
            }
          },
          series: [
            {
              name: 'K线',
              type: 'candlestick',
              data: data.candles,
              itemStyle: {
                color: '#3fb950',
                color0: '#f85149',
                borderColor: '#3fb950',
                borderColor0: '#f85149',
              },
              markLine: {
                symbol: 'none',
                silent: false,
                data: tradeLines,
              },
            },
          ],
        };

        btChart.setOption(option);

        // 自适应大小
        setTimeout(() => btChart.resize(), 100);
        window.addEventListener('resize', () => btChart.resize());

      } catch (e) {
        console.error('加载K线图失败:', e);
      }
    }

    // ====== Top10 精选策略（深度参数探索实测 · 4h/120天 BTCUSDT, 100U, 1x, 复利全仓）======
    // 数据来源：对 ~1600 组网格外参数组合逐一调用 /api/backtest/compound 实测所得
    const TOP10_STRATEGIES = [
      {
        rank: 1, name: 'RSI11 中线 (35.5/55)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 11, bullLevel: 35.5, bearLevel: 55 },
        lookback: 22,
        scanReturn: 103.14, winRate: 85.2, trades: 27, maxDd: 9.12,
        desc: 'RSI11 上穿35.5做多，下穿55做空 · 100U→203U 资金翻倍'
      },
      {
        rank: 2, name: 'RSI11 中线 (36/55)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 11, bullLevel: 36, bearLevel: 55 },
        lookback: 22,
        scanReturn: 100.61, winRate: 84.0, trades: 25, maxDd: 9.12,
        desc: 'RSI11 上穿36做多，下穿55做空'
      },
      {
        rank: 3, name: 'RSI12 中线 (35.5/54.5)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 35.5, bearLevel: 54.5 },
        lookback: 22,
        scanReturn: 100.09, winRate: 81.5, trades: 27, maxDd: 9.05,
        desc: 'RSI12 上穿35.5做多，下穿54.5做空'
      },
      {
        rank: 4, name: 'RSI12 中线 (36/54.5)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 36, bearLevel: 54.5 },
        lookback: 22,
        scanReturn: 100.09, winRate: 81.5, trades: 27, maxDd: 9.05,
        desc: 'RSI12 上穿36做多，下穿54.5做空'
      },
      {
        rank: 5, name: 'RSI12 中线 (34/54.5)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 34, bearLevel: 54.5 },
        lookback: 22,
        scanReturn: 99.08, winRate: 79.2, trades: 24, maxDd: 9.05,
        desc: 'RSI12 上穿34做多，下穿54.5做空'
      },
      {
        rank: 6, name: 'RSI12 中线 (35/54.5)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 35, bearLevel: 54.5 },
        lookback: 22,
        scanReturn: 98.57, winRate: 80.0, trades: 25, maxDd: 9.05,
        desc: 'RSI12 上穿35做多，下穿54.5做空'
      },
      {
        rank: 7, name: 'RSI12 中线 (34.5/54.5)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 34.5, bearLevel: 54.5 },
        lookback: 22,
        scanReturn: 98.02, winRate: 80.0, trades: 25, maxDd: 9.05,
        desc: 'RSI12 上穿34.5做多，下穿54.5做空'
      },
      {
        rank: 8, name: 'RSI12 中线 (37/54.5)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 37, bearLevel: 54.5 },
        lookback: 22,
        scanReturn: 97.60, winRate: 81.5, trades: 27, maxDd: 9.05,
        desc: 'RSI12 上穿37做多，下穿54.5做空'
      },
      {
        rank: 9, name: 'RSI12 中线 (35.5/54)',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '4h',
        params: { period: 12, bullLevel: 35.5, bearLevel: 54 },
        lookback: 22,
        scanReturn: 97.18, winRate: 80.0, trades: 25, maxDd: 9.05,
        desc: 'RSI12 上穿35.5做多，下穿54做空'
      },
      {
        rank: 10, name: 'RSI10 中线 (37.5/55) · 6h',
        kind: 'rsiMidline', strategyId: 'rsi14_midline', interval: '6h',
        params: { period: 10, bullLevel: 37.5, bearLevel: 55 },
        lookback: 20,
        scanReturn: 77.43, winRate: 75.0, trades: 20, maxDd: 10.73,
        desc: '6小时周期冠军：RSI10 上穿37.5做多，下穿55做空'
      },
    ];

    const top10Modal = document.getElementById('top10Modal');
    const top10Body = document.getElementById('top10Body');

    function openTop10Modal() {
      renderTop10();
      top10Modal.classList.add('open');
    }

    function closeTop10Modal() {
      top10Modal.classList.remove('open');
    }

    let lastSweepInterval = '4h';
    let lastSweepDays = 120;

    function openTop20Modal() {
      top20ModalEl.classList.add('open');
      loadTop20();
    }

    function closeTop20Modal() {
      top20ModalEl.classList.remove('open');
    }

    async function loadTop20() {
      const symbol = document.getElementById('symbol').value;
      const interval = document.getElementById('top20Interval').value;
      const days = parseInt(document.getElementById('top20Days').value);
      const top = 20;
      lastSweepInterval = interval;
      lastSweepDays = days;
      const intervalLabel = { '1h':'1小时','2h':'2小时','4h':'4小时','6h':'6小时','8h':'8小时','12h':'12小时','1d':'日线' }[interval] || interval;
      top20Body.innerHTML = `<div class="bt-empty">正在扫描 ${symbol} · ${intervalLabel} · ${days}天，请稍候（正在对全部策略做参数网格遍历）...</div>`;
      try {
        const data = await fetchJson(`${API_BASE}/api/backtest/sweep?symbol=${encodeURIComponent(symbol)}&interval=${interval}&days=${days}&top=${top}`);
        renderTop20(data);
      } catch (e) {
        top20Body.innerHTML = `<div class="bt-empty">扫描失败：${e.message}（请确认后端已启动且数据库有足够 ${symbol} 历史数据）</div>`;
      }
    }

    function renderTop20(data) {
      const rows = data.rows || [];
      const interval = data.interval || lastSweepInterval;
      const days = data.days || lastSweepDays;
      const intervalLabel = { '1h':'1h','2h':'2h','4h':'4h','6h':'6h','8h':'8h','12h':'12h','1d':'1d' }[interval] || interval;
      const barsPerDay = interval === '1d' ? 1 : (24 / parseInt(interval.replace('h','')));
      const actualDays = data.actualDays != null ? data.actualDays : Math.round(data.bars / barsPerDay);
      const fmtTime = (ts) => ts == null ? '—' : new Date(ts).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
      const symbol = document.getElementById('symbol').value;

      let html = `
        <div class="top10-header-info">
          <span>📊 数据周期: <b>${data.symbol} · ${intervalLabel} · 约${actualDays}天</b></span>
          <span>💰 资金: <b>100U · 1x杠杆 · 复利全仓</b></span>
          <span>💸 手续费: <b>0.04%</b></span>
          <span>🔍 样本: <b>${data.bars} 根 ${intervalLabel}</b>（${fmtTime(data.firstOpenTime)} → ${fmtTime(data.lastCloseTime)}）</span>
        </div>
        <div class="catalog-table-wrap">
          <table class="catalog-table">
            <thead>
              <tr>
                <th>排名</th><th>策略</th><th>分类</th><th>参数</th>
                <th>收益率</th><th>胜率</th><th>笔数</th><th>最大回撤</th><th>净收益U</th><th>操作</th>
              </tr>
            </thead>
            <tbody>
      `;

      if (!rows.length) {
        html += `<tr><td colspan="10" style="color:#8b949e;">没有命中任何有效组合（可能数据不足或参数组合均无法成交）。</td></tr>`;
      }

      for (const r of rows) {
        const pnlClass = Number(r.returnPct) >= 0 ? 'catalog-positive' : 'catalog-negative';
        html += `
          <tr data-rank="${r.rank}" style="cursor:pointer;">
            <td>${r.rank}</td>
            <td><div class="catalog-name">${r.name}</div><div class="catalog-desc">${r.category}</div></td>
            <td>${r.category}</td>
            <td style="font-family:monospace;">${r.paramsDesc}</td>
            <td class="${pnlClass}">${fmtNumber(r.returnPct)}%</td>
            <td>${fmtNumber(r.winRate)}%</td>
            <td>${r.trades}</td>
            <td>${fmtNumber(r.maxDd)}%</td>
            <td>${fmtNumber(r.netProfit, 2)}</td>
            <td>
              <button class="top10-run" data-rank="${r.rank}" style="margin-right:4px;">▶ 回测</button>
              <button class="top20-edit" data-rank="${r.rank}" style="background:#21262d; color:#58a6ff; border:1px solid #30363d; border-radius:4px; padding:3px 8px; font-size:11px; cursor:pointer;">✏️ 编辑</button>
            </td>
          </tr>`;
      }

      html += '</tbody></table></div>';
      html += `
        <div style="margin-top:14px; padding:10px 14px; background:#161b22; border:1px solid #30363d; border-radius:8px; font-size:11px; color:#8b949e;">
          ⚠️ 以上为对全部策略「参数网格」遍历后按收益率排序的前 20 名，基于历史数据回测（复利全仓），不构成投资建议。点击行直接运行复利回测；点击「✏️ 编辑」可修改参数后再回测。
        </div>`;

      top20Body.innerHTML = html;

      // 点击行 → 直接运行回测
      top20Body.querySelectorAll('tr[data-rank]').forEach(row => {
        row.addEventListener('click', (e) => {
          if (e.target.tagName === 'BUTTON') return;
          const rank = parseInt(row.dataset.rank);
          const r = rows.find(x => x.rank === rank);
          if (r) runTop20Backtest(r, symbol, interval, days, false);
        });
      });

      // 点击「▶ 回测」按钮 → 直接运行
      top20Body.querySelectorAll('.top10-run').forEach(btn => {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const rank = parseInt(btn.dataset.rank);
          const r = rows.find(x => x.rank === rank);
          if (r) runTop20Backtest(r, symbol, interval, days, false);
        });
      });

      // 点击「✏️ 编辑」按钮 → 打开编辑器
      top20Body.querySelectorAll('.top20-edit').forEach(btn => {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const rank = parseInt(btn.dataset.rank);
          const r = rows.find(x => x.rank === rank);
          if (r) runTop20Backtest(r, symbol, interval, days, true);
        });
      });
    }

    async function runTop20Backtest(r, symbol, interval, days, showEditor) {
      const schema = kindToParamSchema(r.kind);
      const defaultParams = { fast:5, slow:20, useEma:false, period:14, oversold:30, overbought:70,
        bullLevel:50, bearLevel:50, signal:9, k:2.0, threshold:100 };
      const mergedParams = { ...defaultParams, ...r.params };

      const fakeStrategy = {
        id: kindToStrategyId(r.kind),
        name: r.name,
        description: r.category + ' (Top20 #' + r.rank + ')',
        category: kindToCategory(r.kind),
        kind: r.kind,
        lookback: r.lookback,
        paramSchema: schema,
        defaultParams: mergedParams,
        index: r.rank,
      };

      closeTop20Modal();

      if (showEditor) {
        catalogModal.classList.add('open');
        openStrategyEditor(fakeStrategy, {
          compound: true,
          defaultSymbol: symbol,
          defaultInterval: interval,
          defaultDays: days,
          backAction: () => {
            closeCatalogModal();
            openTop20Modal();
          },
        });
      } else {
        catalogModal.classList.add('open');
        const intervalLabel = { '1h':'1小时','2h':'2小时','4h':'4小时','6h':'6小时','8h':'8小时','12h':'12小时','1d':'日线' }[interval] || interval;
        catalogBody.innerHTML = `
          <div style="display:flex; gap:16px; min-height:400px;">
            <div style="flex:0 0 340px; background:#0d1117; border:1px solid #21262d; border-radius:8px; padding:14px;">
              <div style="margin-bottom:12px; display:flex; align-items:center; gap:8px;">
                <button id="spBack2" style="background:#21262d; color:#c9d1d9; border:1px solid #30363d; border-radius:6px; padding:4px 10px; cursor:pointer; font-size:12px;">← 返回</button>
                <span style="color:#8b949e; font-size:12px;">Top20 #${r.rank} · ${intervalLabel}</span>
              </div>
              <h3 style="color:#e6edf3; font-size:15px; margin:0 0 4px 0;">${r.name}</h3>
              <div style="color:#8b949e; font-size:12px; margin-bottom:8px;">${r.paramsDesc}</div>
              <div style="color:#8b949e; font-size:11px; margin-bottom:8px; font-family:monospace;">${symbol} · ${intervalLabel} · ${days}天 · 复利全仓</div>
              <div style="color:#3fb950; font-size:13px; font-weight:600; margin-bottom:8px;">复利回测中...</div>
            </div>
            <div id="spResult" style="flex:1; overflow-y:auto; max-height: calc(100vh - 100px);">
              <div class="bt-empty">正在运行复利回测，请稍候...</div>
            </div>
          </div>
        `;
        document.getElementById('spBack2').addEventListener('click', () => {
          closeCatalogModal();
          openTop20Modal();
        });

        const payload = {
          kind: r.kind,
          symbol: symbol,
          interval: interval,
          days: days,
          lookback: r.lookback,
          capital: 100,
          leverage: 1,
          fee: 0.0004,
          params: r.params,
        };

        try {
          const data = await fetchJson(`${API_BASE}/api/backtest/compound`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
          });
          renderCustomBacktestResult(data);
        } catch(e) {
          const rel = document.getElementById('spResult');
          if (rel) rel.innerHTML = `<div class="bt-empty">复利回测失败：${e.message}</div>`;
        }
      }
    }

    function renderTop10() {
      let html = `
        <div class="top10-header-info">
          <span>📊 数据周期: <b>BTCUSDT · 4h · 最近120天</b></span>
          <span>💰 资金: <b>100U · 1x杠杆 · 复利全仓</b></span>
          <span>💸 手续费: <b>0.04%</b></span>
          <span>🔍 探索: <b>~1600组参数实测 · 网格外深度搜索</b></span>
        </div>
      `;

      for (const s of TOP10_STRATEGIES) {
        let badgeClass = '';
        if (s.rank === 1) badgeClass = 'gold';
        else if (s.rank === 2) badgeClass = 'silver';
        else if (s.rank === 3) badgeClass = 'bronze';

        const paramsStr = Object.entries(s.params).map(([k,v]) => `${k}=${v}`).join(', ');
        const intervalTag = s.interval && s.interval !== '4h'
          ? `<span style="background:#1f6feb33; color:#58a6ff; border-radius:3px; padding:1px 5px; font-size:10px; margin-left:6px;">${s.interval}</span>` : '';

        html += `
          <div class="top10-card" data-rank="${s.rank}">
            <div class="top10-badge ${badgeClass}">${s.rank}</div>
            <div class="top10-info">
              <div class="top10-name">${s.name}${intervalTag}</div>
              <div class="top10-desc">${s.desc} · ${paramsStr} · LB=${s.lookback}</div>
            </div>
            <div class="top10-stats">
              <div class="top10-stat">
                <div class="val">+${s.scanReturn.toFixed(1)}%</div>
                <div class="lbl">收益率</div>
              </div>
              <div class="top10-stat">
                <div class="val" style="color:#c9d1d9; font-size:13px;">${s.winRate.toFixed(0)}%</div>
                <div class="lbl">胜率</div>
              </div>
              <div class="top10-stat">
                <div class="val" style="color:#f0b429; font-size:13px;">${s.trades}</div>
                <div class="lbl">笔数</div>
              </div>
              <div class="top10-stat">
                <div class="val neg" style="font-size:13px;">${s.maxDd.toFixed(1)}%</div>
                <div class="lbl">回撤</div>
              </div>
            </div>
            <div style="display:flex; gap:6px; flex-shrink:0;">
              <button class="top10-run" data-rank="${s.rank}">▶ 打开</button>
            </div>
          </div>
        `;
      }

      html += `
        <div style="margin-top:14px; padding:10px 14px; background:#161b22; border:1px solid #30363d; border-radius:8px; font-size:11px; color:#8b949e;">
          ⚠️ 以上收益率基于历史数据回测（复利全仓模式），不构成投资建议。点击卡片或「▶ 打开」进入策略编辑界面，可调整参数后运行回测。
        </div>
      `;

      top10Body.innerHTML = html;

      // 卡片点击 → 进入策略编辑界面
      top10Body.querySelectorAll('.top10-card').forEach(card => {
        card.addEventListener('click', (e) => {
          if (e.target.tagName === 'BUTTON') return;
          const rank = parseInt(card.dataset.rank);
          const s = TOP10_STRATEGIES.find(x => x.rank === rank);
          if (s) runTop10Backtest(s, true);
        });
      });
      // 打开按钮 → 进入策略编辑界面
      top10Body.querySelectorAll('.top10-run').forEach(btn => {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const rank = parseInt(btn.dataset.rank);
          const s = TOP10_STRATEGIES.find(x => x.rank === rank);
          if (s) runTop10Backtest(s, true);
        });
      });
    }

    function kindToStrategyId(kind) {
      // Map kind to a real strategy ID from the catalog
      const map = {
        'maCross': 'sma_7_25',
        'rsiMidline': 'rsi14_midline',
        'rsiReversal': 'rsi7_reversal',
        'rsiLongOnly': 'rsi14_long_20_90',
        'macdCross': 'macd_12_26_9',
        'bollReversion': 'boll20_reversion',
        'bollBreakout': 'boll20_breakout',
        'kdjCross': 'kdj9_cross',
        'cciReversal': 'cci20_reversal',
        'cciMidline': 'cci20_midline',
        'priceMaCross': 'price_ma20',
        'donchianBreakout': 'donchian20',
        'rsiTakeProfit': 'rsi14_tp_20_90',
      };
      return map[kind] || 'sma_7_25';
    }

    function kindToParamSchema(kind) {
      // Build schema matching the backend's param definitions
      switch(kind) {
        case 'maCross':
          return [
            { key: 'fast', label: '快线周期', type: 'int', min: 2, max: 200, step: 1 },
            { key: 'slow', label: '慢线周期', type: 'int', min: 3, max: 500, step: 1 },
            { key: 'useEma', label: '使用 EMA', type: 'bool' },
          ];
        case 'rsiMidline':
          return [
            { key: 'period', label: 'RSI 周期', type: 'int', min: 2, max: 100, step: 1 },
            { key: 'bullLevel', label: '多头阈值', type: 'float', min: 0, max: 60, step: 0.1 },
            { key: 'bearLevel', label: '空头阈值', type: 'float', min: 40, max: 100, step: 0.1 },
          ];
        case 'rsiReversal':
          return [
            { key: 'period', label: 'RSI 周期', type: 'int', min: 2, max: 100, step: 1 },
            { key: 'oversold', label: '超卖阈值', type: 'float', min: 0, max: 50, step: 0.1 },
            { key: 'overbought', label: '超买阈值', type: 'float', min: 50, max: 100, step: 0.1 },
          ];
        case 'rsiLongOnly':
          return [
            { key: 'period', label: 'RSI 周期', type: 'int', min: 2, max: 100, step: 1 },
            { key: 'oversold', label: '开多阈值', type: 'float', min: 0, max: 50, step: 0.1 },
            { key: 'overbought', label: '平多阈值', type: 'float', min: 50, max: 100, step: 0.1 },
          ];
        case 'macdCross':
          return [
            { key: 'fast', label: '快线 EMA', type: 'int', min: 2, max: 100, step: 1 },
            { key: 'slow', label: '慢线 EMA', type: 'int', min: 3, max: 200, step: 1 },
            { key: 'signal', label: '信号线', type: 'int', min: 2, max: 100, step: 1 },
          ];
        case 'bollReversion':
        case 'bollBreakout':
          return [
            { key: 'period', label: '布林周期', type: 'int', min: 5, max: 100, step: 1 },
            { key: 'k', label: '带宽倍数 k', type: 'float', min: 0.5, max: 5, step: 0.1 },
          ];
        case 'kdjCross':
          return [{ key: 'period', label: 'KDJ 周期', type: 'int', min: 2, max: 100, step: 1 }];
        case 'cciReversal':
          return [
            { key: 'period', label: 'CCI 周期', type: 'int', min: 2, max: 100, step: 1 },
            { key: 'threshold', label: '极值阈值', type: 'float', min: 50, max: 300, step: 1 },
          ];
        case 'cciMidline':
          return [{ key: 'period', label: 'CCI 周期', type: 'int', min: 2, max: 100, step: 1 }];
        case 'priceMaCross':
          return [
            { key: 'period', label: 'MA 周期', type: 'int', min: 2, max: 200, step: 1 },
            { key: 'useEma', label: '使用 EMA', type: 'bool' },
          ];
        case 'donchianBreakout':
          return [{ key: 'period', label: '通道周期', type: 'int', min: 3, max: 200, step: 1 }];
        case 'rsiTakeProfit':
          return [
            { key: 'period', label: 'RSI 周期', type: 'int', min: 2, max: 100, step: 1 },
            { key: 'oversold', label: '开多阈值(RSI上穿)', type: 'float', min: 0, max: 50, step: 0.1 },
            { key: 'overbought', label: '开空阈值(RSI上穿)', type: 'float', min: 50, max: 100, step: 0.1 },
          ];
        default:
          return [];
      }
    }

    function kindToCategory(kind) {
      const map = {
        'maCross': '均线交叉', 'rsiReversal': 'RSI反转', 'rsiMidline': 'RSI中线',
        'rsiLongOnly': 'RSI做多', 'macdCross': 'MACD', 'bollReversion': '布林回归', 'bollBreakout': '布林突破',
        'kdjCross': 'KDJ', 'cciReversal': 'CCI反转', 'cciMidline': 'CCI中线',
        'priceMaCross': '价格/MA', 'donchianBreakout': '唐奇安突破', 'rsiTakeProfit': 'RSI止盈',
      };
      return map[kind] || '其他';
    }

    async function runTop10Backtest(s, showEditor) {
      // Build a fake strategy object compatible with openStrategyEditor
      const schema = kindToParamSchema(s.kind);
      const defaultParams = { fast:5, slow:20, useEma:false, period:14, oversold:30, overbought:70,
        bullLevel:50, bearLevel:50, signal:9, k:2.0, threshold:100 };
      // Merge s.params on top of defaults
      const mergedParams = { ...defaultParams, ...s.params };

      const fakeStrategy = {
        id: s.strategyId,
        name: s.name,
        description: s.desc + ' (Top10 精选 #' + s.rank + ')',
        category: kindToCategory(s.kind),
        kind: s.kind,
        lookback: s.lookback,
        paramSchema: schema,
        defaultParams: mergedParams,
        index: s.rank,
      };

      // Close top10 modal, open catalog modal with the strategy editor
      closeTop10Modal();

      // 让 catalogModal 全屏（Top10 场景）
      const catalogInner = catalogModal.querySelector('.modal');
      const origStyle = catalogInner.style.cssText;
      catalogInner.style.cssText = 'width: 100vw; height: 100vh; max-width: 100vw; max-height: 100vh; border-radius: 0; margin: 0;';
      const restoreCatalog = () => { catalogInner.style.cssText = origStyle; };

      if (showEditor) {
        // Show compound-mode parameter editor
        catalogModal.classList.add('open');
        openStrategyEditor(fakeStrategy, {
          compound: true,
          defaultInterval: s.interval || '4h',
          defaultDays: 120,
          backAction: () => {
            restoreCatalog();
            closeCatalogModal();
            openTop10Modal();
          },
        });
      } else {
        // Run directly without showing editor — use compound backtest (same as sweep)
        catalogModal.classList.add('open');
        catalogBody.innerHTML = `
          <div style="display:flex; gap:16px; min-height:400px; height: calc(100vh - 100px);">
            <div style="flex:0 0 340px; background:#0d1117; border:1px solid #21262d; border-radius:8px; padding:14px; overflow-y:auto;">
              <div style="margin-bottom:12px; display:flex; align-items:center; gap:8px;">
                <button id="spBack2" style="background:#21262d; color:#c9d1d9; border:1px solid #30363d; border-radius:6px; padding:4px 10px; cursor:pointer; font-size:12px;">← 返回</button>
                <span style="color:#8b949e; font-size:12px;">Top10 精选 #${s.rank}</span>
              </div>
              <h3 style="color:#e6edf3; font-size:15px; margin:0 0 4px 0;">${s.name}</h3>
              <div style="color:#8b949e; font-size:12px; margin-bottom:8px;">${s.desc}</div>
              <div style="color:#8b949e; font-size:11px; margin-bottom:8px; font-family:monospace;">BTCUSDT · ${s.interval || '4h'} · 120天 · 复利全仓</div>
              <div style="color:#3fb950; font-size:13px; font-weight:600; margin-bottom:8px;">复利回测中...</div>
            </div>
            <div id="spResult" style="flex:1; overflow-y:auto;">
              <div class="bt-empty">正在运行复利回测，请稍候...</div>
            </div>
          </div>
        `;
        document.getElementById('spBack2').addEventListener('click', () => {
          restoreCatalog();
          closeCatalogModal();
          openTop10Modal();
        });

        const payload = {
          kind: s.kind,
          symbol: 'BTCUSDT',
          interval: s.interval || '4h',
          days: 120,
          lookback: s.lookback,
          capital: 100,
          leverage: 1,
          fee: 0.0004,
          params: s.params,
        };

        try {
          const data = await fetchJson(`${API_BASE}/api/backtest/compound`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
          });
          renderCustomBacktestResult(data);
        } catch(e) {
          const rel = document.getElementById('spResult');
          if (rel) rel.innerHTML = `<div class="bt-empty">复利回测失败：${e.message}</div>`;
        }
      }
    }

    function openBacktestModal() {
      syncBacktestFormWithChart();
      backtestModal.classList.add('open');
      runStrategyCatalog();
    }

    function buildBacktestPayload() {
      return {
        symbol: document.getElementById('btSymbol').value,
        interval: document.getElementById('btInterval').value,
        fast_ma: Number(document.getElementById('btFastMa').value),
        slow_ma: Number(document.getElementById('btSlowMa').value),
        lookback: Number(document.getElementById('btLookback').value),
        limit: Number(document.getElementById('btLimit').value),
        capital: Number(document.getElementById('btCapital').value),
        quantity: Number(document.getElementById('btQuantity').value),
        leverage: Number(document.getElementById('btLeverage').value),
        fee: Number(document.getElementById('btFee').value),
      };
    }

    // 毫秒时间戳 → 可读时间（无值显示占位）
    function fmtTime(ts) {
      if (ts == null) return '—';
      const d = new Date(ts);
      return d.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
    }

    async function openDbModal() {
      dbModal.classList.add('open');
      dbBody.textContent = '加载中...';
      try {
        const data = await fetchJson(`${API_BASE}/api/db`);
        const rows = data.rows || [];
        if (rows.length === 0) {
          dbBody.innerHTML = '<div class="db-empty">数据库暂无数据</div>';
          return;
        }
        let html = '<table class="db-table"><thead><tr><th>合约</th><th>周期</th><th>K线数量</th><th>最早</th><th>最新</th></tr></thead><tbody>';
        for (const r of rows) {
          html += `<tr><td>${r.symbol}</td><td>${r.interval}</td><td>${r.count}</td><td>${fmtTime(r.earliest)}</td><td>${fmtTime(r.latest)}</td></tr>`;
        }
        html += '</tbody></table>';
        dbBody.innerHTML = html;
      } catch (e) {
        dbBody.innerHTML = `<div class="db-empty">加载失败：${e.message}（请确认后端已启动）</div>`;
      }
    }

    async function runBacktest() {
      const payload = buildBacktestPayload();

      backtestBody.innerHTML = '<div class="bt-empty">回测中...</div>';
      try {
        const data = await fetchJson(`${API_BASE}/api/backtest`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });
        renderBacktestResult(data);
        void runStrategyCatalog();
      } catch (e) {
        backtestBody.innerHTML = `<div class="bt-empty">回测失败：${e.message}</div>`;
      }
    }

    async function runStrategyCatalog() {
      const payload = buildBacktestPayload();
      backtestCatalog.innerHTML = '<div class="bt-empty">100 个常见策略批量回测中...</div>';
      try {
        const data = await fetchJson(`${API_BASE}/api/backtest/catalog`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });
        renderBacktestCatalogResults(data);
      } catch (e) {
        backtestCatalog.innerHTML = `<div class="bt-empty">批量回测失败：${e.message}</div>`;
      }
    }

    function renderBacktestResult(data) {
      const params = data.parameters || {};
      const summary = data.summary || {};
      const trades = data.recentTrades || [];
      const profitClass = Number(summary.netProfit) >= 0 ? 'positive' : 'negative';

      let html = `
        <div class="bt-meta">
          <span class="bt-pill">${params.symbol || '—'} · ${params.interval || '—'}</span>
          <span class="bt-pill">MA${params.fastMa}/${params.slowMa}</span>
          <span class="bt-pill">Lookback ${params.lookback ?? '—'}</span>
          <span class="bt-pill">样本 ${summary.bars ?? 0} 根</span>
          <span class="bt-pill">时间 ${fmtTime(summary.firstOpenTime)} → ${fmtTime(summary.lastCloseTime)}</span>
        </div>
        <div class="bt-summary">
          <div class="bt-card"><span class="label">净收益</span><span class="value ${profitClass}">${fmtNumber(summary.netProfit)}</span></div>
          <div class="bt-card"><span class="label">收益率</span><span class="value ${profitClass}">${fmtNumber(summary.returnPct)}%</span></div>
          <div class="bt-card"><span class="label">最终权益</span><span class="value">${fmtNumber(summary.finalEquity)}</span></div>
          <div class="bt-card"><span class="label">最大回撤</span><span class="value">${fmtNumber(summary.maxDrawdownPct)}%</span></div>
          <div class="bt-card"><span class="label">胜率</span><span class="value">${fmtNumber(summary.winRatePct)}%</span></div>
          <div class="bt-card"><span class="label">交易笔数</span><span class="value">${summary.tradeCount ?? 0}</span></div>
        </div>
        <div class="bt-meta">
          <span class="bt-pill">初始资金 ${fmtNumber(params.capital)}U</span>
          <span class="bt-pill">单次保证金 ${fmtNumber(params.marginPerTrade ?? params.quantity, 2)}U</span>
          <span class="bt-pill">倍数 ${fmtNumber(params.leverage, 2)}</span>
          <span class="bt-pill">单次名义仓位 ${fmtNumber(params.notionalPerTrade, 2)}U</span>
          <span class="bt-pill">手续费率 ${fmtNumber(params.feeRate, 4)}</span>
          <span class="bt-pill">盈利 ${summary.winCount ?? 0}</span>
          <span class="bt-pill">亏损 ${summary.lossCount ?? 0}</span>
          <span class="bt-pill">总手续费 ${fmtNumber(summary.totalFees, 4)}</span>
        </div>
      `;

      if (!trades.length) {
        html += '<div class="bt-empty">本次回测没有触发任何成交。</div>';
        backtestBody.innerHTML = html;
        return;
      }

      html += '<div class="bt-table-wrap"><table class="bt-table"><thead><tr><th>方向</th><th>入场时间</th><th>入场价</th><th>出场时间</th><th>出场价</th><th>净收益</th><th>手续费</th><th>持有根数</th></tr></thead><tbody>';
      for (const trade of trades) {
        html += `<tr>
          <td>${trade.side}</td>
          <td>${fmtTime(trade.entryTime)}</td>
          <td>${fmtNumber(trade.entryPrice, 2)}</td>
          <td>${fmtTime(trade.exitTime)}</td>
          <td>${fmtNumber(trade.exitPrice, 2)}</td>
          <td>${fmtNumber(trade.netPnl, 2)}</td>
          <td>${fmtNumber(trade.fee, 4)}</td>
          <td>${trade.barsHeld}</td>
        </tr>`;
      }
      html += '</tbody></table></div>';
      backtestBody.innerHTML = html;
    }

    function renderBacktestCatalogResults(data) {
      const params = data.parameters || {};
      const strategies = data.strategies || [];

      if (!strategies.length) {
        backtestCatalog.innerHTML = '<div class="bt-empty">没有拿到策略回测结果。</div>';
        return;
      }

      let html = `
        <div class="bt-meta">
          <span class="bt-pill">${params.symbol || '—'} · ${params.interval || '—'}</span>
          <span class="bt-pill">样本 ${params.limit ?? '全部'} 根</span>
          <span class="bt-pill">初始资金 ${fmtNumber(params.capital)}U</span>
          <span class="bt-pill">单次保证金 ${fmtNumber(params.marginPerTrade ?? params.quantity, 2)}U</span>
          <span class="bt-pill">倍数 ${fmtNumber(params.leverage, 2)}</span>
          <span class="bt-pill">单次名义仓位 ${fmtNumber(params.notionalPerTrade, 2)}U</span>
          <span class="bt-pill">手续费率 ${fmtNumber(params.feeRate, 4)}</span>
        </div>
        <div class="catalog-table-wrap">
          <table class="catalog-table">
            <thead>
              <tr>
                <th>排名</th>
                <th>策略</th>
                <th>分类</th>
                <th>Lookback</th>
                <th>交易笔数</th>
                <th>胜率</th>
                <th>净收益</th>
                <th>收益率</th>
                <th>最大回撤</th>
              </tr>
            </thead>
            <tbody>
      `;

      for (const strategy of strategies) {
        const pnlClass = Number(strategy.netProfit) >= 0 ? 'catalog-positive' : 'catalog-negative';
        html += `
          <tr>
            <td>${strategy.rank}</td>
            <td>
              <div class="catalog-name">${strategy.name}</div>
              <div class="catalog-desc">${strategy.description}</div>
            </td>
            <td>${strategy.category}</td>
            <td>${strategy.lookback}</td>
            <td>${strategy.tradeCount}</td>
            <td>${fmtNumber(strategy.winRatePct)}%</td>
            <td class="${pnlClass}">${fmtNumber(strategy.netProfit)}</td>
            <td class="${pnlClass}">${fmtNumber(strategy.returnPct)}%</td>
            <td>${fmtNumber(strategy.maxDrawdownPct)}%</td>
          </tr>
        `;
      }

      html += '</tbody></table></div>';
      backtestCatalog.innerHTML = html;
    }
