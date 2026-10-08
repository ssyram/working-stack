/* ================= 引用编辑/预览系统 v2（三路审查修复） ================= */
const goldbox=document.getElementById('goldbox'), ghost=document.getElementById('ghost'),
      refmenu=document.getElementById('refmenu'), chip=document.getElementById('chip'),
      bubble=document.getElementById('bubble'), pvov=document.getElementById('pvov');
let REF_DRAFT=null;

function pidNum(pidStr){ const n=parseInt(String(pidStr).split('-').pop(),10); return isNaN(n)?null:n; }
function frameByIdNum(n){ return STATE ? STATE.stack.frames[String(n)] : null; }

/* ---------- 路径分词：把用户输入拆为 {base, filter}（F1 修复核心） ---------- */
function refTokenize(input) {
  const s = String(input||'');
  if (!s) return {base:'', filter:'', drill:false};
  if (s.endsWith('.')) return {base:s.slice(0,-1), filter:'', drill:true};
  if (s.endsWith('[')) return {base:s.slice(0,-1), filter:'', drill:true};
  const lastDot = s.lastIndexOf('.');
  const lastBracket = s.lastIndexOf(']');
  if (lastDot < 0 && lastBracket < 0) return {base:'', filter:s, drill:false};
  const splitAt = Math.max(lastDot, lastBracket);
  let base, filter;
  if (s[splitAt] === '.') { base = s.slice(0, splitAt); filter = s.slice(splitAt + 1); }
  else { base = s.slice(0, splitAt + 1); filter = s.slice(splitAt + 1); }
  return {base, filter, drill:false};
}

/* ---------- 路径文法（完整路径，严格验证——F11 修复） ---------- */
function refParse(path){
  const segs=[]; let rest=String(path||'').trim();
  if(!rest) return segs;
  const headEnd=rest.search(/[.[]/); const head=headEnd<0?rest:rest.slice(0,headEnd);
  if(head.startsWith('note:')){
    const key=head.slice(5);
    if(!key) return null;
    segs.push({t:'note',k:key});
  } else {
    const tail=head.split('-').pop();
    if(!tail || !/^\d+$/.test(tail)) return null;
    segs.push({t:'f',n:parseInt(tail,10)});
  }
  rest=headEnd<0?'':rest.slice(headEnd);
  while(rest){
    if(rest[0]==='.'){
      const e=rest.slice(1).search(/[.[]/); const f=e<0?rest.slice(1):rest.slice(1,1+e);
      if(!f) return null;
      segs.push({t:'fld',f});
      rest=e<0?'':rest.slice(1+e);
    } else if(rest[0]==='['){
      const e=rest.indexOf(']');
      if(e<0) return null;
      const idx=rest.slice(1,e);
      if(!/^\d+$/.test(idx)) return null;
      segs.push({t:'ix',i:parseInt(idx,10)});
      rest=rest.slice(e+1);
    } else return null;
  }
  return segs;
}

/* ---------- 客户端解析（F5 修复：匹配 Rust resolve.rs 语义） ---------- */
function refResolve(path){
  const segs=refParse(path);
  if(segs===null) return {err:'语法'};
  if(segs.length===0) return {err:'空路径'};
  let frameId=null, noteKey=null, f=null, n=null, list=null, listKind=null, item=null;

  for(const s of segs){
    if(s.t==='f'){
      frameId=s.n; f=frameByIdNum(s.n);
      if(!f) return {err:'帧不存在'};
    } else if(s.t==='note'){
      noteKey=s.k; n=STATE.stack.notes[s.k];
      if(!n) return {err:'note 不存在'};
    } else if(s.t==='fld'){
      // log[i] 后的字段（detail/at/action）——在主循环内处理
      if(listKind==='log' && item && ['detail','at','action'].includes(s.f)){
        if(s.f==='detail') return {ok:true,text:flat(item.detail),frameId};
        if(s.f==='at') return {ok:true,text:item.at,frameId};
        if(s.f==='action') return {ok:true,text:String(item.action).toUpperCase(),frameId};
      }
      if(f){
        switch(s.f){
          case 'title': return {ok:true,text:f.title,frameId};
          case 'goal': return {ok:true,text:(f.goal||[]).map(g=>flat(g)).join('\n\n'),frameId};
          case 'origin': return {ok:true,text:(f.origin||[]).map(g=>flat(g)).join('\n\n'),frameId};
          case 'status':
            if(f.status.kind==='paused') return {ok:true,text:'paused：'+flat(f.status.reason||''),frameId};
            if(f.status.kind==='done'||f.status.kind==='expired'){
              return {ok:true,text:f.status.kind+'：'+flat(f.status.closure?.summary||''),frameId};
            }
            return {ok:true,text:'active',frameId};
          case 'plan': list=f.plan||[]; listKind='plan'; item=null; break;
          case 'log': list=f.log||[]; listKind='log'; item=null; break;
          case 'results': list=(f.content&&f.content.results)||[]; listKind='results'; item=null; break;
          case 'side_effects': list=(f.content&&f.content.side_effects)||[]; listKind='side_effects'; item=null; break;
          case 'principles': list=(f.content&&f.content.principles)||[]; listKind='principles'; item=null; break;
          case 'remarks':
            const rms=f.content&&f.content.remarks||{};
            const vals=Object.entries(rms).map(([k,v])=>k+'：'+flat(v));
            return {ok:true,text:vals.join('\n')||'（无备注）',frameId};
          default: return {err:'字段 '+s.f};
        }
      } else if(n){
        if(s.f==='body') return {ok:true,text:flat(n.body)};
        if(s.f==='meta') { list=null; listKind='meta'; item=n.meta; break; }
        return {err:'字段 '+s.f};
      } else if(listKind==='meta' && item){
        if(s.f==='kind') return {ok:true,text:item.kind||'note'};
        if(s.f==='shared') return {ok:true,text:String(item.shared||false)};
        return {err:'字段 '+s.f};
      } else return {err:'此处无字段'};
    } else if(s.t==='ix'){
      if(!list || listKind===null) return {err:'此处无下标'};
      item=list[s.i];
      if(!item) return {err:'下标 ['+s.i+'] 越界（共 '+(list||[]).length+' 项）'};
      if(listKind==='plan'){
        // plan[i] 返回项正文（Rust 语义）
        // 后续还可跟 .text？不——Rust plan[i] 直接返回 text
        return {ok:true,text:flat(item.text),frameId};
      } else if(listKind==='log'){
        // log[i] 可后续 .detail/.at/.action
        // 如果这是最后一段，返回摘要
        if(segs.indexOf(s)===segs.length-1){
          return {ok:true,text:'['+String(item.action).toUpperCase()+'] ['+item.at+'] '+flat(item.detail),frameId};
        }
        // 否则继续处理后续字段
      } else if(listKind==='results'||listKind==='side_effects'){
        return {ok:true,text:flat(item),frameId};
      } else if(listKind==='principles'){
        return {ok:true,text:flat(item.rule),frameId};
      }
    }
  }

  // 处理 log[i].detail/.at/.action 的后续字段
  // 重新遍历（简化：在 ix 分支之后找 fld）
  const lastIx = segs.map(s=>s.t).lastIndexOf('ix');
  if(lastIx>=0 && segs.length>lastIx+1){
    const fldSeg=segs[lastIx+1];
    if(fldSeg.t==='fld' && listKind==='log' && item){
      if(fldSeg.f==='detail') return {ok:true,text:flat(item.detail),frameId};
      if(fldSeg.f==='at') return {ok:true,text:item.at,frameId};
      if(fldSeg.f==='action') return {ok:true,text:String(item.action).toUpperCase(),frameId};
      return {err:'字段 '+fldSeg.f};
    }
  }

  if(f && frameId!=null) return {ok:true,text:'['+pidOf(frameId)+' '+f.title+']',frameId};
  if(n) return {ok:true,text:flat(n.body)};
  if(list){
    if(listKind==='plan') return {ok:true,text:'计划 '+(list||[]).length+' 项（用 [i] 下标访问）',frameId};
    if(listKind==='log') return {ok:true,text:'记录 '+(list||[]).length+' 条（用 [i] 下标访问）',frameId};
  }
  return {err:'未终结路径'};
}

/* ---------- 候选生成（F1 修复：基于 refTokenize 分词） ---------- */
function refCandidates(input){
  if(!STATE||!STATE.stack) return {opts:[]};
  const {base, filter, drill} = refTokenize(input);

  // 根级：空 base
  if(!base){
    const opts=[];
    Object.keys(STATE.stack.frames).map(Number).sort((a,b)=>a-b).forEach(n=>{
      const f=STATE.stack.frames[String(n)];
      const label=pidOf(n);
      opts.push({ins:label,label:label,desc:trunc(f.title,24),sub:true,
        pv:{frameId:n,text:'['+label+' '+f.title+']\n'+(f.goal||[]).map(flat).join(' ').slice(0,100)}});
    });
    Object.keys(STATE.stack.notes||{}).sort().forEach(k=>{
      const n=STATE.stack.notes[k];
      opts.push({ins:'note:'+k,label:'note:'+k,desc:(n.meta&&n.meta.kind||'note')+' · '+trunc(flat(n.body),20),sub:true,
        pv:{frameId:null,text:'['+k+']\n'+trunc(flat(n.body),140)}});
    });
    const fl=filter.toLowerCase();
    return {opts:opts.filter(o=>o.label.toLowerCase().includes(fl)||o.desc.toLowerCase().includes(fl))};
  }

  // 有 base：解析确定段
  const bsegs = refParse(base);
  if(bsegs===null) return {opts:[],err:'基路径语法错误'};

  // 确定 base 指向什么类型
  const lastSeg = bsegs[bsegs.length-1];
  const opts=[];

  // 如果 base 是一个帧 ID
  if(bsegs.length===1 && bsegs[0].t==='f'){
    const f=frameByIdNum(bsegs[0].n);
    if(!f) return {opts:[]};
    addFieldOpts(opts, base, bsegs, f);
  }
  // 如果 base 是 note:key
  else if(bsegs.length===1 && bsegs[0].t==='note'){
    const n=STATE.stack.notes[bsegs[0].k];
    if(!n) return {opts:[]};
    opts.push({ins:'.body',label:'.body',desc:'正文',sub:false,
      pv:{frameId:null,text:trunc(flat(n.body),140)}});
    opts.push({ins:'.meta',label:'.meta',desc:'元信息',sub:true,
      pv:{frameId:null,text:JSON.stringify(n.meta||{})}});
  }
  // 如果 base 以 .meta 结尾（note 下钻）
  else if(bsegs.length===3 && bsegs[0].t==='note' && bsegs[2] && bsegs[2].t==='fld' && bsegs[2].f==='meta'){
    opts.push({ins:'.kind',label:'.kind',desc:'类别',sub:false,pv:{frameId:null,text:'kind'}});
    opts.push({ins:'.shared',label:'.shared',desc:'共享',sub:false,pv:{frameId:null,text:'shared'}});
  }
  // 如果 base 以 .plan / .log / .results / .side_effects / .principles 结尾
  else if(bsegs.length>=2 && lastSeg.t==='fld' && ['plan','log','results','side_effects','principles'].includes(lastSeg.f)){
    const fid=bsegs[0].t==='f'?bsegs[0].n:null;
    const f=fid!=null?frameByIdNum(fid):null;
    let list=[];
    if(lastSeg.f==='plan') list=(f&&f.plan)||[];
    else if(lastSeg.f==='log') list=(f&&f.log)||[];
    else if(f&&f.content) list=(f.content[lastSeg.f==='results'?'results':lastSeg.f==='side_effects'?'side_effects':'principles'])||[];
    list.forEach((it,i)=>{
      const txt=lastSeg.f==='plan'?flat(it.text)
        :lastSeg.f==='log'?('['+String(it.action).toUpperCase()+'] '+it.at)
        :lastSeg.f==='principles'?flat(it.rule)
        :flat(it);
      opts.push({ins:'['+i+']',label:'['+i+']',desc:trunc(txt,28),sub:false,
        pv:{frameId:fid,text:'['+base+'['+i+']]\n'+trunc(txt,140)}});
    });
  }
  // 如果 base 以 [i] 结尾且前一个字段是 log
  else if(lastSeg.t==='ix'){
    const prevFld=bsegs.find(s=>s.t==='fld'&&s.f==='log');
    if(prevFld){
      opts.push({ins:'.detail',label:'.detail',desc:'内容',sub:false,pv:{frameId:null,text:'事件内容'}});
      opts.push({ins:'.at',label:'.at',desc:'时间',sub:false,pv:{frameId:null,text:'时间戳'}});
      opts.push({ins:'.action',label:'.action',desc:'行为',sub:false,pv:{frameId:null,text:'行为类型'}});
    }
  }

  const fl=filter.toLowerCase();
  const filtered=opts.filter(o=>o.label.toLowerCase().includes(fl)||o.desc.toLowerCase().includes(fl));
  return {opts:filtered};
}

function addFieldOpts(opts, base, bsegs, f){
  const fid=bsegs[0].n;
  const mkPv=(fld)=>{
    const r=refResolve(base+'.'+fld);
    return {frameId:fid,text:r.ok?'['+(base+'.'+fld)+']\n'+trunc(r.text,140):(r.err||'')};
  };
  opts.push({ins:'.title',label:'.title',desc:'标题',sub:false,pv:mkPv('title')});
  opts.push({ins:'.goal',label:'.goal',desc:'弹帧目标',sub:false,pv:mkPv('goal')});
  opts.push({ins:'.origin',label:'.origin',desc:'来源',sub:false,pv:mkPv('origin')});
  opts.push({ins:'.status',label:'.status',desc:'状态',sub:false,pv:mkPv('status')});
  if(f.plan&&f.plan.length) opts.push({ins:'.plan',label:'.plan',desc:'计划 '+(f.plan||[]).length+' 项',sub:true,pv:mkPv('plan')});
  if(f.log&&f.log.length) opts.push({ins:'.log',label:'.log',desc:'记录 '+(f.log||[]).length+' 条',sub:true,pv:mkPv('log')});
  if(f.content){
    if(f.content.results&&f.content.results.length) opts.push({ins:'.results',label:'.results',desc:'结果 '+(f.content.results||[]).length+' 项',sub:true,pv:mkPv('results')});
    if(f.content.side_effects&&f.content.side_effects.length) opts.push({ins:'.side_effects',label:'.side_effects',desc:'副作用 '+(f.content.side_effects||[]).length+' 项',sub:true,pv:mkPv('side_effects')});
    if(f.content.principles&&f.content.principles.length) opts.push({ins:'.principles',label:'.principles',desc:'原则 '+(f.content.principles||[]).length+' 条',sub:true,pv:mkPv('principles')});
    if(f.content.remarks&&Object.keys(f.content.remarks).length) opts.push({ins:'.remarks',label:'.remarks',desc:'备注',sub:false,pv:mkPv('remarks')});
  }
}

/* ---------- 菜单交互 ---------- */
let REF_ACTIVE=-1, REF_OPTS=[];
function refOpen(el){
  REF_DRAFT={el};
  const txt=editText(el), caret=caretPos(el);
  const before=txt.slice(0,caret);
  const m=before.lastIndexOf('{{');
  if(m<0||before.slice(m).includes('}}')){ refClose(); return; }
  const escOK=before.slice(0,m).endsWith('\\');
  if(escOK){ refClose(); return; }
  REF_DRAFT.start=m+2;
  REF_DRAFT.typed=before.slice(REF_DRAFT.start);
  refUpdate();
}

function refUpdate(){
  try{
    if(!REF_DRAFT) return;
    const c=refCandidates(REF_DRAFT.typed);
    REF_OPTS=c.opts||[];
    REF_ACTIVE=REF_OPTS.length?0:-1;
    chipHide();
    if(!REF_OPTS.length){
      // 不静默关闭——显示零候选原因（U14）
      refmenu.style.display='block';
      refmenu.innerHTML='<div class="mhead">引用</div><div class="ri" style="color:#8b949e;padding:8px 12px;font-size:11px">无候选'+(c.err?'（'+esc(c.err)+'）':'')+'<br>检查路径或按 Esc 关闭</div>';
      const r=caretRect(REF_DRAFT.el);
      refmenu.style.left=Math.min(r.x, innerWidth-380)+'px';
      refmenu.style.top=Math.min(r.bottom+6, innerHeight-280)+'px';
      return;
    }
    refmenu.style.display='block';
    refmenu.innerHTML='<div class="mhead">引用：Tab/Enter 确定 · . 下钻 · Esc 关闭</div>'+REF_OPTS.map(function(o,i){
      return '<div class="ri'+(i===0?' active':'')+'" data-i="'+i+'"><span class="code">'+esc(o.label)+'</span><span class="desc">'+esc(o.desc||'')+(o.sub?' ·':'')+'</span></div>';
    }).join('');
    const r=caretRect(REF_DRAFT.el);
    refmenu.style.left=Math.min(r.x, innerWidth-380)+'px';
    refmenu.style.top=Math.min(r.bottom+6, innerHeight-280)+'px';
    [...refmenu.querySelectorAll('.ri')].forEach(div=>{
      const i=+div.dataset.i;
      div.onmouseenter=()=>{ refActivate(i); };
      div.onclick=()=>{ refConfirm(i); };
    });
    if(REF_ACTIVE>=0) refPreview(REF_OPTS[0]);
  }catch(err){ console.error('[ws-ref] refUpdate:', err); refClose(); }
}

function refActivate(i){
  REF_ACTIVE=i;
  [...refmenu.querySelectorAll('.ri')].forEach((d,j)=>d.classList.toggle('active',j===i));
  refPreview(REF_OPTS[i]);
  const el=refmenu.querySelectorAll('.ri')[i];
  if(el) el.scrollIntoView({block:'nearest'});
}

function refPreview(o){
  if(!o||!o.pv){ goldHide(); ghostHide(); return; }
  if(o.pv.frameId!=null){
    const node=d3.selectAll('.node').filter(e=>e.id===o.pv.frameId).node();
    if(node){ const b=node.getBoundingClientRect(); goldTo(b); ghostAt(b.right+12, b.top, o.pv.text); return; }
  }
  const mb=refmenu.getBoundingClientRect();
  goldTo(mb); ghostAt(mb.right+10, mb.top, o.pv.text);
}

function refConfirm(i){
  const o=REF_OPTS[i]; if(!o||!REF_DRAFT) return;
  const el=REF_DRAFT.el;
  const txt=editText(el), caret=caretPos(el);
  // 用候选 ins 替换 filter 部分：从 start 到 caret 替换为 base + ins
  const {base}=refTokenize(REF_DRAFT.typed);
  const newSeg = base ? base + o.ins : o.ins;
  const newTxt = txt.slice(0,REF_DRAFT.start) + newSeg + txt.slice(caret);
  setEditText(el,newTxt);
  const nc = REF_DRAFT.start + newSeg.length;
  setCaret(el,nc);
  REF_DRAFT.typed = newSeg;

  if(!o.sub){
    insertAtCaret(el,'}}');
    refClose(); chipHide();
    playPreview(newSeg);
  } else {
    // 子级候选：追加 '.' 触发下钻（否则 refCandidates 会把 typed 当过滤词卡在根层）
    insertAtCaret(el, '.');
    REF_DRAFT.typed = newSeg + '.';
    refUpdate();
  }
}

function refClose(){ refmenu.style.display='none'; REF_OPTS=[]; REF_ACTIVE=-1; goldHide(); ghostHide(); }
function chipShow(){
  if(!REF_DRAFT) return;
  const r=caretRect(REF_DRAFT.el);
  chip.style.display='flex'; chip.style.left=r.x+'px'; chip.style.top=(r.bottom+6)+'px';
}
function chipHide(){ chip.style.display='none'; }
chip.querySelectorAll('button').forEach(b=>b.onclick=()=>{
  if(!REF_DRAFT) return;
  insertAtCaret(REF_DRAFT.el, b.dataset.ins);
  if(b.dataset.ins==='}}'){
    const t=REF_DRAFT.typed; refClose(); playPreview(t);
  } else {
    REF_DRAFT.typed+='.';
    chipHide(); refUpdate();
  }
});

/* ---------- 编辑光标工具（F3 修复：统一换行处理） ---------- */
function editText(el){
  if(el.isContentEditable){
    // 统一：innerText 与 Range toString 都把 <br> 读为 \n，但可能有差异
    // 用 selection API 的 toString 作为权威
    return el.innerText.replace(/\u00a0/g,' ');
  }
  return el.value;
}
function setEditText(el,t){
  if(el.isContentEditable){ el.innerText=t; }
  else { el.value=t; }
}
function caretPos(el){
  if(el.isContentEditable){
    const s=getSelection();
    if(s.rangeCount&&el.contains(s.anchorNode)){
      const r=document.createRange();
      r.selectNodeContents(el);
      r.setEnd(s.anchorNode,s.anchorOffset);
      // Range.toString() 不含 <br> 的换行，与 innerText 可能不一致
      // 修复：统一用 textContent（不含 <br> 换行）做计数
      return r.toString().replace(/\u00a0/g,' ').length;
    }
    return el.innerText.replace(/\u00a0/g,' ').length;
  }
  return el.selectionStart ?? el.value.length;
}
function setCaret(el,p){
  el.focus();
  if(el.isContentEditable){
    const s=getSelection(),r=document.createRange();
    let node=el.firstChild, off=p;
    while(node){
      if(node.nodeType===3){
        if(off<=node.textContent.length){ r.setStart(node,off); r.collapse(true); s.removeAllRanges(); s.addRange(r); return; }
        off-=node.textContent.length;
      } else if(node.nodeType===1){ // <br> 等
        off=Math.max(0,off-1);
      }
      node=node.nextSibling;
    }
    r.selectNodeContents(el); r.collapse(false); s.removeAllRanges(); s.addRange(r);
  }
  else el.setSelectionRange(p,p);
}
function insertAtCaret(el,txt){
  const p=caretPos(el); const t=editText(el);
  setEditText(el,t.slice(0,p)+txt+t.slice(p)); setCaret(el,p+txt.length);
}
function caretRect(el){
  if(el.isContentEditable){
    const s=getSelection();
    if(s.rangeCount){
      const r=s.getRangeAt(0).getBoundingClientRect();
      if(r.width||r.height) return {x:r.left,bottom:r.bottom};
    }
  }
  const b=el.getBoundingClientRect(); return {x:b.left,bottom:b.bottom};
}

/* ---------- 金框 + 虚影 + 画布平移 ---------- */
function goldTo(b){
  goldbox.style.left=(b.left-4)+'px'; goldbox.style.top=(b.top-4)+'px';
  goldbox.style.width=(b.width+8)+'px'; goldbox.style.height=(b.height+8)+'px';
  goldbox.classList.add('on');
}
function goldHide(){ goldbox.classList.remove('on'); }
function ghostAt(x,y,html){
  ghost.innerHTML='<div class="gtitle">预览</div>'+esc(html);
  ghost.classList.add('on');
  ghost.style.left=Math.min(x+8,innerWidth-360)+'px';
  ghost.style.top=Math.min(y,innerHeight-240)+'px';
}
function ghostHide(){ ghost.classList.remove('on'); }

/* ---------- 键盘路由（F2 修复：preventDefault + F4 修复：Esc 分层） ---------- */
document.addEventListener('keydown', e=>{
  try{
    // 最上层：弹窗 → 不拦截（原生处理）
    if(document.querySelector('.modal-mask')) return;

    // 第二层：预览框
    if(pvov.style.display==='flex'){
      if(e.key==='Escape'){ e.stopPropagation(); pvClose(); return; }
      return;
    }

    // 第三层：引用菜单
    if(refmenu.style.display==='block'){
      if(e.key==='ArrowDown'||e.key==='ArrowUp'){ e.preventDefault(); refActivate(Math.max(0,Math.min(REF_OPTS.length-1,REF_ACTIVE+(e.key==='ArrowDown'?1:-1)))); return; }
      if(e.key==='Tab'||e.key==='Enter'){ e.preventDefault(); refConfirm(REF_ACTIVE); return; }
      if(e.key==='Escape'){ e.preventDefault(); e.stopPropagation(); refClose(); return; }
      if(e.key==='.'&&REF_DRAFT){ e.preventDefault(); insertAtCaret(REF_DRAFT.el,'.'); REF_DRAFT.typed+='.'; setTimeout(()=>refUpdate(),0); return; }
    }

    // 第四层：右键菜单
    if(document.querySelector('.ctx')){
      if(e.key==='Escape'){ e.preventDefault(); e.stopPropagation(); document.querySelectorAll('.ctx').forEach(x=>x.remove()); return; }
    }

    // 第五层：pick 模式
    if(PICKING && e.key==='Escape'){
      e.stopPropagation(); stopPick(); return;
    }

    // 最底层：图视图（只在无上层交互时执行）
    if(e.key==='Escape'){
      LOCKED=null;SELECTED=null;EDITING=false;
      clearHl(); draw(); showSide();
    }
  }catch(err){ console.error('[ws-ref] keydown:', err); }
}, true);

/* ---------- 输入监听（F3 修复：keyup 备份 + refable 限定） ---------- */
function onRefInput(e){
  try{
    const el=e.target;
    if(!(el&&(el.isContentEditable||['INPUT','TEXTAREA'].includes(el.tagName)))) return;
    // U10：仅 refable 类的字段触发引用（args JSON/reason/note key 不劫持）
    if(!el.classList.contains('refable')) return;
    const txt=editText(el), caret=caretPos(el);
    const before=txt.slice(0,caret);
    if(before.endsWith('{{')&&!before.slice(0,-2).endsWith('\\')){
      setTimeout(()=>{try{refOpen(el);}catch(err){console.error('[ws-ref] refOpen:',err);}},0);
    }
    else if(REF_DRAFT&&REF_DRAFT.el===el){
      REF_DRAFT.typed=before.slice(REF_DRAFT.start);
      if(before.length<=REF_DRAFT.start) refClose();
      else refUpdate();
    }
  }catch(err){ console.error('[ws-ref] input:', err); }
}
document.addEventListener('input', onRefInput, true);
document.addEventListener('keyup', e=>{
  if(e.key==='{'||e.key==='.'){ onRefInput(e); }
}, true);

/* ---------- 校验 + 转义气泡 ---------- */
function refValidateText(txt){
  const problems=[];
  const noEsc=txt.replace(/\\\{\{|\\\}\}/g,'');
  const re=/\{\{([^}]*)\}\}|(\}\})/g; let m;
  while((m=re.exec(noEsc))){
    if(m[2]) problems.push({kind:'stray'});
    else if(!m[1]||refParse(m[1])===null) problems.push({kind:'bad',txt:m[1]});
    else { const r=refResolve(m[1]); if(r.err) problems.push({kind:'unres',txt:m[1],err:r.err}); }
  }
  const open=(noEsc.match(/\{\{/g)||[]).length-(noEsc.match(/\}\}/g)||[]).length;
  if(open>0) problems.push({kind:'unclosed'});
  return problems;
}
function showBubble(anchor, msg, onFix){
  bubble.style.display='block';
  bubble.innerHTML=esc(msg)+'<br><button data-a="fix">{{ → \\{{</button><button data-a="ok">忽略</button>';
  const b=anchor.getBoundingClientRect();
  bubble.style.left=Math.min(b.left, innerWidth-330)+'px'; bubble.style.top=(b.bottom+8)+'px';
  bubble.querySelector('[data-a="fix"]').onclick=()=>{ bubble.style.display='none'; if(onFix)onFix(); };
  bubble.querySelector('[data-a="ok"]').onclick=()=>{ bubble.style.display='none'; };
}
function escapeRefs(el){
  const t=editText(el);
  setEditText(el, t.replace(/\{\{/g,'\\{{').replace(/\}\}/g,'\\}}'));
}

/* ---------- 路径播放预览（含全帧预览卡 + 画布平移） ---------- */
let PV_TIMER=null;
function playPreview(path){
  const r=refResolve(path); const segs=refParse(path)||[];
  pvov.style.display='flex';
  document.getElementById('pvref').textContent='{{'+path+'}}';
  const steps=document.getElementById('pvsteps');
  let acc='';
  steps.innerHTML=segs.map(s=>{
    acc+=s.t==='f'?pidOf(s.n):s.t==='note'?'note:'+s.k:s.t==='fld'?'.'+s.f:'['+s.i+']';
    return '<span class="step" data-p="'+esc(acc)+'">'+esc(acc)+'</span>';
  }).join('');
  const body=document.getElementById('pvbody');
  body.textContent='';
  const stepEls=[...steps.querySelectorAll('.step')];
  let i=0; clearInterval(PV_TIMER);

  const step=()=>{
    stepEls.forEach((el,j)=>el.classList.toggle('lit',j<=i));
    const p=stepEls[i].dataset.p, rr=refResolve(p);
    body.textContent='{{'+p+'}}\n'+(rr.ok?rr.text:'⟨'+(rr.err||'未解析')+'⟩');
    if(rr.frameId!=null){
      const node=d3.selectAll('.node').filter(e=>e.id===rr.frameId).node();
      if(node){
        const b=node.getBoundingClientRect(); goldTo(b);
        ghostAt(b.right+12,b.top,rr.ok?trunc(rr.text,140):'');
        // 画布平移到该帧（用户需求：金色框时画布移到左半屏）
        panToNode(rr.frameId);
      }
    }
    i++;
    if(i>=stepEls.length){ clearInterval(PV_TIMER); }
  };
  step();
  if(stepEls.length>1) PV_TIMER=setInterval(step,650);
}

/* 画布平移：将目标帧移到视口左半边（用户需求） */
function panToNode(frameId){
  const pos=NODE_POS[frameId];
  if(!pos) return;
  const svgEl=svg.node(); if(!svgEl) return;
  const box=svgEl.getBoundingClientRect();
  const scale=Math.max(0.5,Math.min(1.2,box.width/(CARD_W*6)));
  const tx=box.width*0.25-pos.x*scale;
  const ty=box.height*0.4-pos.y*scale;
  svg.transition().duration(600).call(zoom.transform,d3.zoomIdentity.translate(tx,ty).scale(scale));
}

function pvClose(){ pvov.style.display='none'; clearInterval(PV_TIMER); goldHide(); ghostHide(); }
document.getElementById('pvclose').onclick=pvClose;
document.getElementById('pvsave').onclick=()=>{
  const refTxt=document.getElementById('pvref').textContent.replace(/^\{\{|\}\}$/g,'');
  const r=refResolve(refTxt);
  const finalText=r.ok?r.text:'预览失败';
  showModal({title:'保存预览为 note（最终结果）', fields:[{id:'key',label:'note key',value:'preview-'+Date.now().toString(36),required:true}], onOK:v=>{
    doOp('note',{key:v.key,body:'{{'+refTxt+'}} = '+finalText},'保存引用预览');
  }});
};
document.getElementById('pvBtn').onclick=()=>{
  showModal({title:'全局引用预览（只读）', fields:[{id:'ref',label:'表达式（如 1.goal 或 note:k.body）',required:true}], okText:'播放',
    onOK:v=>{
      if(refParse(v.ref)===null||refParse(v.ref).length===0){ return; }
      playPreview(v.ref);
    }});
};

/* ---------- 右键复制引用 / plan 右键菜单（F7 修复：data-ref 渲染） ---------- */
document.addEventListener('contextmenu', e=>{
  const t=e.target.closest('[data-ref]');
  if(!t) return;
  e.preventDefault(); e.stopPropagation();
  document.querySelectorAll('.ctx').forEach(x=>x.remove());
  const ref=t.dataset.ref;
  const menu=document.createElement('div'); menu.className='ctx';
  const items=[{t:'📋 复制引用 {{'+ref+'}}',fn:()=>navigator.clipboard.writeText('{{'+ref+'}}')}];
  if(t.dataset.planPath){
    items.push({t:'＋ 添加内嵌子项…',fn:()=>{
      showModal({title:'内嵌子项 → '+t.dataset.planPath, fields:[{id:'text',label:'内容',required:true}], onOK:v=>{
        doOp('update',{id:pidOf(SELECTED),plan_add_child:[t.dataset.planPath+'='+v.text]},'右键添加内嵌列表');
      }});
    }});
  }
  items.forEach(it=>{
    const d=document.createElement('div'); d.className='ci';
    d.innerHTML='<span>'+esc(it.t)+'</span>';
    d.onclick=()=>{menu.remove();it.fn();};
    menu.appendChild(d);
  });
  menu.style.left=Math.min(e.clientX,innerWidth-260)+'px';
  menu.style.top=Math.min(e.clientY,innerHeight-140)+'px';
  document.body.appendChild(menu);
  const close=()=>{menu.remove();document.removeEventListener('click',close);};
  setTimeout(()=>document.addEventListener('click',close),0);
});
