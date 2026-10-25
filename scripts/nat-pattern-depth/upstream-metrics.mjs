// SPDX-License-Identifier: MPL-2.0
import {pathToFileURL} from 'node:url';
const B=await import(pathToFileURL(process.argv[2]).href);const book=B.book_nil();await B.book_load(book,process.argv[3],'',new Map());B.book_valid(book);
for(const name of process.argv.slice(4)){
 const d=book.tlds[name];if(!d?.v)throw Error('missing '+name);
 for(const [stage,h] of [['loaded',d.v],['normalized',B.term_snf(book,d.v)]]){
  const body=B.term_lower(h);const stack=[[body,0,[]]];let depth=0,nodes=0,path=[],tags={};
  while(stack.length){const [t,n,p]=stack.pop();nodes++;tags[t.$]=(tags[t.$]??0)+1;if(n>depth){depth=n;path=p;}
   const keys={All:['A','B'],Lam:['f'],App:['f','x'],Mat:['h','m'],Typ:['g'],Min:['a','b'],Eql:['a','b','T'],Rwt:['e','p','f'],Ann:['x','T'],Sub:['v','f'],Let:['v','f'],Ctr:['x'],ADT:['x']}[t.$]??[];
   for(const k of keys)for(const [i,ch] of (Array.isArray(t[k])?t[k]:[t[k]]).entries())if(ch?.$)stack.push([ch,n+1,[...p,t.$+'.'+k+'.'+i]]);
  }
  console.log(JSON.stringify({name,stage,depth,nodes,tags,path,body,canonical:canonical(body,[])}));
 }
}
function canonical(t,b){switch(t.$){case 'Var':{const i=b.toReversed().indexOf(t.i);if(i<0)throw Error('unbound');return `Var:${i};`;}case 'Ref':return `Ref:${t.k};`;case 'Lam':return 'Lam;'+canonical(t.f,[...b,t.i]);case 'App':return 'App;'+canonical(t.f,b)+canonical(t.x,b);case 'Mat':return `Mat:${t.k};`+canonical(t.h,b)+canonical(t.m,b);case 'Ctr':return `Ctr:${t.k}:${t.x.length};`+t.x.map(x=>canonical(x,b)).join('');case 'Efq':return 'Efq;';case 'Ann':return 'Ann;'+canonical(t.x,b)+canonical(t.T,b);case 'ADT':return `Adt:${t.k}:${t.x.length};`+t.x.map(x=>canonical(x,b)).join('');default:throw Error('unsupported '+t.$);}}
